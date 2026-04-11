use std::{collections::HashMap, sync::Arc};

use axum::{
    Json, Router,
    body::Body,
    extract::{Path, State},
    http::{StatusCode, header},
    response::Html,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tokio::net::TcpListener;

async fn group_name(Path(group_id): Path<u32>, State(state): State<Arc<AppState>>) -> Json<String> {
    let mut conn = state.db.acquire().await.unwrap();

    match sqlx::query!("select name from groups where id = ?1", group_id)
        .fetch_one(&mut *conn)
        .await
    {
        Ok(rec) => Json(rec.name),
        Err(e) => panic!("{e:?}"),
    }
}
async fn group_members(
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

async fn tasks(Path(group_id): Path<u32>, State(state): State<Arc<AppState>>) -> Json<HashMap<u32, String>> {
    let mut conn = state.db.acquire().await.unwrap();

    let rec = sqlx::query!("select id, title from tasks where gr = ?1", group_id).fetch_all(&mut *conn).await.unwrap();

    Json(rec.into_iter().map(|a| (a.id as u32, a.title)).collect())
}
async fn task_title(Path(task_id): Path<u32>, State(state): State<Arc<AppState>>) -> Json<String> {
    let mut conn = state.db.acquire().await.unwrap();

    let rec = sqlx::query!("select title from tasks where id = ?1", task_id).fetch_one(&mut *conn).await.unwrap();

    Json(rec.title)
}

#[derive(Serialize, Deserialize)]
enum TaskType {
    Assignment,
    Social,
    StudyHall,
    Reading,
}

#[derive(Serialize, Deserialize)]
struct TaskCreationData {
    title: String,
    r#type: Option<TaskType>,
    due: Option<String>,
    reward: i32,
}
async fn create_task(Path(group_id): Path<u32>, Json(data): Json<TaskCreationData>) -> Json<()> {
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
    ];
    let mut router = Router::new();
    for (path, real_path) in static_routes {
        let data: &'static str = String::leak(std::fs::read_to_string(
            "frontend/static/".to_owned() + real_path,
        )?);
        router = router.route(path, get(move || async move { Html(data) }));
    }

    router = router
        .route("/api/group/{group_id}/name", get(group_name))
        .route("/api/group/{group_id}/members", get(group_members))
        .route("/api/group/{group_id}/task", get(tasks))
        .route("/api/group/{group_id}/task", post(create_task))
        .route("/api/task/{task_id}/title", get(task_title));
        // .route("/api/task/{task_id}", delete(delete_task));

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("Starting at http://0.0.0.0:3000");
    axum::serve(listener, router.with_state(Arc::new(app_state))).await?;

    Ok(())
}
