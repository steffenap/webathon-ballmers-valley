use std::sync::Arc;

use axum::{
    Json, Router,
    routing::{delete, get, post},
};
use serde::Deserialize;
use sqlx::SqlitePool;
use tokio::net::TcpListener;

pub(crate) mod jwt;

pub type UserId = u32;

mod api {
    mod stupid_imports {
        pub(crate) use crate::AppState;
        pub(crate) use axum::{
            Json,
            extract::{Path, State},
        };
        pub(crate) use serde::Deserialize;
        pub(crate) use std::{collections::HashMap, sync::Arc};
    }

    use axum_extra::extract::cookie;
    use stupid_imports::*;

    pub(crate) async fn cookies(cookie_jar: axum_extra::extract::CookieJar) -> Json<()> {
        println!("All cookies:");
        for cookie in cookie_jar.iter() {
            println!("{cookie:?}");
        }
        Json(())
    }

    pub(crate) mod user {
        use std::time::{Instant, UNIX_EPOCH};

        use axum::{http::StatusCode, response::Redirect};
        use axum_extra::extract::{CookieJar, cookie::Cookie};
        use serde::Serialize;

        use crate::{UserId, api::group::stuff::UserData, jwt::Authenticator};

        use super::stupid_imports::*;
        pub(crate) async fn groups(
            Path(user_id): Path<u32>,
            State(state): State<Arc<AppState>>,
        ) -> Json<HashMap<u32, String>> {
            let mut conn = state.db.acquire().await.unwrap();

            let res = sqlx::query!(
                "select id, name from groups inner join user_group on user_group.gr = groups.id and user_group.user = ?1", user_id
            ).fetch_all(&mut *conn).await.unwrap();

            Json(res.into_iter().map(|x| (x.id as u32, x.name)).collect())
        }

        #[derive(Serialize)]
        pub(crate) struct UserDetails {
            id: u32,
            name: String,
            full_name: String,
            health: u32,
        }

        pub(crate) async fn user_details(
            Path(user_id): Path<u32>,
            State(state): State<Arc<AppState>>,
        ) -> Json<UserDetails> {
            let mut conn = state.db.acquire().await.unwrap();

            let res = sqlx::query!(
                "select id, username, full_name, health from users where id = ?1",
                user_id
            )
            .fetch_one(&mut *conn)
            .await
            .unwrap();

            Json(UserDetails {
                id: res.id as u32,
                name: res.username,
                full_name: res.full_name,
                health: res.health as u32,
            })
        }

        pub(crate) async fn whoami(
            State(state): State<Arc<AppState>>,
            jar: CookieJar,
        ) -> Result<Json<UserId>, Redirect> {
            if let Some(cookie) = crate::jwt::Cookie::from_jar(&jar)
                && let Some(ans) = state.auth.validate(&cookie)
            {
                Ok(Json(ans))
            } else {
                Err(Redirect::temporary("/login"))
            }
        }

        #[derive(Deserialize)]
        pub(crate) struct SignupData {
            username: String,
            fullname: String,
            password: String,
        }

        pub(crate) async fn signup(
            State(state): State<Arc<AppState>>,
            jar: CookieJar,
            Json(data): Json<SignupData>,
        ) -> (CookieJar, Json<()>) {
            let mut conn = state.db.acquire().await.unwrap();
            let elapsed = UNIX_EPOCH.elapsed().unwrap().as_secs() as i64;
            let rec = sqlx::query!(
                "insert into users (username, password, full_name, health, health_last_tick) values (?1, ?2, ?3, ?4, ?5) returning id", data.username, data.password, data.fullname, 1000, elapsed
            ).fetch_one(&mut *conn).await.unwrap();

            let mut cookie = axum_extra::extract::cookie::Cookie::from(
                state.auth.authenticate(rec.id as UserId),
            );
            cookie.set_path("/");
            (jar.add(cookie), Json(()))
        }

        #[derive(Deserialize)]
        pub(crate) struct SigninData {
            username: String,
            password: String,
        }

        #[axum::debug_handler]
        pub(crate) async fn signin(
            State(state): State<Arc<AppState>>,
            jar: CookieJar,
            Json(SigninData { username, password }): Json<SigninData>,
        ) -> (CookieJar, Json<()>) {
            let mut conn = state.db.acquire().await.unwrap();

            let rec = sqlx::query!(
                "select id from users where username = ?1 and password = ?2",
                username,
                password
            )
            .fetch_one(&mut *conn)
            .await
            .unwrap();

            let mut cookie = axum_extra::extract::cookie::Cookie::from(
                state.auth.authenticate(rec.id as UserId),
            );
            cookie.set_path("/");
            (jar.add(cookie), Json(()))
        }
    }

    pub(crate) mod group {
        use super::stupid_imports::*;

        pub(crate) mod stuff {
            use serde::Serialize;

            use super::super::stupid_imports::*;
            #[derive(Serialize)]
            pub(crate) struct TaskData {
                title: String,
                r#type: Option<String>,
                due: Option<u64>,
                reward: u32,
            }

            impl TaskData {
                fn new(
                    title: String,
                    r#type: Option<String>,
                    due: Option<u64>,
                    reward: u32,
                ) -> Self {
                    TaskData {
                        title,
                        r#type,
                        due,
                        reward,
                    }
                }
            }
            #[derive(Serialize)]
            pub(crate) struct UserData {
                id: u32,
                name: String,
                full_name: String,
                health: u32,
            }
            #[derive(Serialize)]
            pub(crate) struct GroupData {
                name: String,
                admin_id: u32,
                users: Vec<UserData>,
                tasks: Vec<TaskData>,
            }

            #[axum::debug_handler]
            pub(crate) async fn group_details(
                Path(group_id): Path<u32>,
                State(state): State<Arc<AppState>>,
            ) -> Json<GroupData> {
                let mut conn = state.db.acquire().await.unwrap();

                let user_res = sqlx::query!("select users.id, username, full_name, health from users inner join user_group on users.id = user_group.user where user_group.gr = ?1",
                    group_id
                )
                .fetch_all(&mut *conn)
                .await.unwrap();

                let task_res = sqlx::query!(
                    "select title, type, due, reward from tasks where gr = ?1",
                    group_id
                )
                .fetch_all(&mut *conn)
                .await
                .unwrap();

                let group_data = sqlx::query!("select name from groups where id = ?1", group_id)
                    .fetch_one(&mut *conn)
                    .await
                    .unwrap();

                let tasks = task_res
                    .into_iter()
                    .map(|a| {
                        TaskData::new(a.title, a.r#type, a.due.map(|x| x as u64), a.reward as u32)
                    })
                    .collect::<Vec<TaskData>>();
                let users = user_res
                    .into_iter()
                    .map(|a| UserData {
                        id: a.id as u32,
                        name: a.username,
                        full_name: a.full_name,
                        health: a.health as u32,
                    })
                    .collect::<Vec<UserData>>();

                Json(GroupData {
                    name: group_data.name,
                    admin_id: 0,
                    users,
                    tasks,
                })
            }
        }

        #[derive(Deserialize)]
        pub(crate) struct GroupInviteData {
            user_id: u32,
        }

        pub(crate) async fn invite(
            Path(group_id): Path<u32>,
            State(state): State<Arc<AppState>>,
            Json(invite_info): Json<GroupInviteData>,
        ) -> Json<()> {
            let mut conn = state.db.acquire().await.unwrap();

            sqlx::query!(
                "insert into user_group(user, gr) values (?1, ?2)",
                group_id,
                invite_info.user_id
            )
            .execute(&mut *conn)
            .await
            .unwrap();

            Json(())
        }
        #[derive(Deserialize)]
        pub(crate) struct GroupCreationData {
            title: String,
        }
        pub(crate) async fn create(
            State(state): State<Arc<AppState>>,
            Json(group_info): Json<GroupCreationData>,
        ) -> Json<()> {
            let mut conn = state.db.acquire().await.unwrap();

            sqlx::query!("insert into groups(name) values (?1)", group_info.title)
                .execute(&mut *conn)
                .await
                .unwrap();

            Json(())
        }

        pub(crate) async fn delete(
            Path(group_id): Path<u32>,
            State(state): State<Arc<AppState>>,
        ) -> Json<()> {
            let mut conn = state.db.acquire().await.unwrap();

            sqlx::query!("delete from groups where id = ?1", group_id)
                .execute(&mut *conn)
                .await
                .unwrap();

            // Json(())
            Json(())
        }

        pub(crate) async fn name(
            Path(group_id): Path<u32>,
            State(state): State<Arc<AppState>>,
        ) -> Json<String> {
            let mut conn = state.db.acquire().await.unwrap();

            match sqlx::query!("select name from groups where id = ?1", group_id)
                .fetch_one(&mut *conn)
                .await
            {
                Ok(rec) => Json(rec.name),
                Err(e) => panic!("{e:?}"),
            }
        }
        pub(crate) async fn members(
            Path(group_id): Path<u32>,
            State(state): State<Arc<AppState>>,
        ) -> Json<HashMap<u32, String>> {
            let mut conn = state.db.acquire().await.unwrap();

            match sqlx::query!("select users.id, username from users inner join user_group on users.id = user_group.user where user_group.gr = ?1",
                group_id
            )
            .fetch_all(&mut *conn)
            .await
            {
                Ok(rec) => Json(rec.into_iter().map(|a| (a.id as u32, a.username)).collect()),
                Err(e) => panic!("{e:?}"),
            }
        }
        pub(crate) async fn tasks(
            Path(group_id): Path<u32>,
            State(state): State<Arc<AppState>>,
        ) -> Json<HashMap<u32, String>> {
            let mut conn = state.db.acquire().await.unwrap();

            let rec = sqlx::query!("select id, title from tasks where gr = ?1", group_id)
                .fetch_all(&mut *conn)
                .await
                .unwrap();

            Json(rec.into_iter().map(|a| (a.id as u32, a.title)).collect())
        }
    }

    pub(crate) mod task {
        use axum_extra::extract::CookieJar;

        use super::stupid_imports::*;
        pub(crate) async fn title(
            Path(task_id): Path<u32>,
            State(state): State<Arc<AppState>>,
        ) -> Json<String> {
            let mut conn = state.db.acquire().await.unwrap();

            let rec = sqlx::query!("select title from tasks where id = ?1", task_id)
                .fetch_one(&mut *conn)
                .await
                .unwrap();

            Json(rec.title)
        }

        #[derive(Deserialize, Copy, Clone)]
        pub(crate) enum TaskType {
            Assignment,
            Social,
            StudyHall,
            Reading,
        }

        impl TaskType {
            fn to_str(self) -> &'static str {
                match self {
                    TaskType::Assignment => "Assignment",
                    TaskType::Social => "Social",
                    TaskType::StudyHall => "StudyHall",
                    TaskType::Reading => "Reading",
                }
            }
        }

        #[derive(Deserialize)]
        pub(crate) struct TaskCreationData {
            title: String,
            r#type: Option<TaskType>,
            due: Option<u64>,
            reward: i32,
            group: Option<u32>,
        }
        pub(crate) async fn create(
            State(state): State<Arc<AppState>>,
            jar: CookieJar,
            Json(data): Json<TaskCreationData>,
        ) -> Json<()> {
            let mut conn = state.db.acquire().await.unwrap();
            let TaskCreationData {
                title,
                r#type,
                due,
                reward,
                group,
            } = data;

            let r#type = r#type.map(|x| x.to_str());
            let due = due.map(|x| x as i64);

            sqlx::query!(
                "insert into tasks(title, type, due, reward, gr) values (?1, ?2, ?3, ?4, ?5)",
                title,
                r#type,
                due,
                reward,
                group,
            )
            .execute(&mut *conn)
            .await
            .unwrap();

            Json(())
        }

        pub(crate) async fn delete(
            Path(task_id): Path<u32>,
            State(state): State<Arc<AppState>>,
        ) -> Json<()> {
            let mut conn = state.db.acquire().await.unwrap();

            sqlx::query!("delete from tasks where id = ?1", task_id)
                .execute(&mut *conn)
                .await
                .unwrap();

            Json(())
        }
    }
}

struct AppState {
    db: SqlitePool,
    auth: jwt::Authenticator,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let db = SqlitePool::connect("db.db").await?;
    let auth = jwt::Authenticator::new();
    let app_state = AppState { db, auth };

    let static_routes = [
        ("/", "index.html"),
        ("/login", "login.html"),
        ("/style.css", "style.css"),
        ("/tasks", "tasks.html"),
        ("/assignments", "assignments.html"),
        ("/group", "group.html"),
        ("/utils.js", "utils.js"),
        ("/images/grinder.png", "images/grinder.png"),
        ("/register", "register.html"),
    ];
    let mut router = Router::new();
    for (path, real_path) in static_routes {
        let data: &'static [u8] =
            Vec::leak(std::fs::read("frontend/static/".to_owned() + real_path)?);
        let content_type = match std::path::Path::new(real_path)
            .extension()
            .map(|x| x.to_str().unwrap())
        {
            Some("html") => "text/html; charset=utf-8",
            Some("js") => "application/javascript",
            Some("png") => "image/png",
            _ => "",
        };
        router = router.route(
            path,
            get(async move || ([("content-type", content_type)], data)),
        );
    }

    router = router
        .route("/api/group/{group_id}/name", get(api::group::name))
        .route("/api/group/{group_id}/members", get(api::group::members))
        .route("/api/group/{group_id}/task", get(api::group::tasks))
        .route(
            "/api/group/{group_id}",
            delete(api::group::delete).get(api::group::stuff::group_details),
        )
        .route("/api/group", post(api::group::create))
        .route("/api/group/{group_id}/invite", post(api::group::invite))
        .route("/api/user/{user_id}/groups", get(api::user::groups))
        .route("/api/user/signup", post(api::user::signup))
        .route("/api/user/whoami", get(api::user::whoami))
        .route("/api/cookies", get(api::cookies))
        .route("/api/user/{user_id}", get(api::user::user_details))
        .route("/api/user/signin", post(api::user::signin))
        .route("/api/task/{task_id}", delete(api::task::delete))
        .route("/api/task", post(api::task::create))
        .route("/api/task/{task_id}/title", get(api::task::title));

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("Starting at http://0.0.0.0:3000");
    axum::serve(listener, router.with_state(Arc::new(app_state))).await?;

    Ok(())
}
