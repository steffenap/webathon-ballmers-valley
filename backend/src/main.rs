use axum::{Router, response::Html, routing::get};
use tokio::net::TcpListener;

async fn index() -> Html<&'static str> {
    Html(include_str!("../../frontend/src/index.html"))
}
async fn login() -> Html<&'static str> {
    Html(include_str!("../../frontend/src/login.html"))
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let router = Router::new()
        .route("/", get(index))
        .route("/login", get(login));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
