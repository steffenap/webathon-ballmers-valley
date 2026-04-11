use std::{collections::HashMap, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tokio::net::TcpListener;

mod api {
    mod stupid_imports {
        pub(crate) use crate::AppState;
        pub(crate) use axum::{
            Json, Router,
            extract::{Path, State},
            routing::{delete, get, post},
        };
        pub(crate) use serde::Deserialize;
        pub(crate) use std::{collections::HashMap, sync::Arc};
    }

    pub(crate) mod user {
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
    }

    pub(crate) mod group {
        use super::stupid_imports::*;

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

            match sqlx::query!("select users.id, name from users inner join user_group on users.id = user_group.user where user_group.gr = ?1",
                group_id
            )
            .fetch_all(&mut *conn)
            .await
            {
                Ok(rec) => Json(rec.into_iter().map(|a| (a.id as u32, a.name)).collect()),
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

#[derive(Deserialize)]
struct LoginData {
    username: String,
    password: String,
}

async fn login(Json(login_data): Json<LoginData>) -> Json<()> {
    Json(())
}

struct AppState {
    db: SqlitePool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let db = SqlitePool::connect("db.db").await?;
    let app_state = AppState { db };

    let static_routes = [
        ("/", "index.html"),
        ("/login", "login.html"),
        ("/style.css", "style.css"),
        ("/tasks", "tasks.html"),
        ("/assignments", "assignments.html"),
        ("/group", "group.html"),
        ("/utils.js", "utils.js"),
        ("/images/grinder.png", "images/grinder.png"),
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
            get(move || async move { ([("content-type", content_type)], data) }),
        );
    }

    router = router
        .route("/api/group/{group_id}/name", get(api::group::name))
        .route("/api/group/{group_id}/members", get(api::group::members))
        .route("/api/group/{group_id}/task", get(api::group::tasks))
        .route("/api/group/{group_id}", delete(api::group::delete))
        .route("/api/group", post(api::group::create))
        .route("/api/group/{group_id}/invite", post(api::group::invite))
        .route("/api/user/{user_id}/groups", get(api::user::groups))
        .route("/api/task/{task_id}", delete(api::task::delete))
        .route("/api/task", post(api::task::create))
        .route("/api/task/{task_id}/title", get(api::task::title))
    // keep semicolon below
    ;

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("Starting at http://0.0.0.0:3000");
    axum::serve(listener, router.with_state(Arc::new(app_state))).await?;

    Ok(())
}
