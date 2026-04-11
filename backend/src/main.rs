use axum::{
    Router,
    http::{StatusCode, header},
    response::Html,
    routing::get,
};
use tokio::net::TcpListener;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let static_routes = [
        ("/",            "index.html"),
        ("/login",       "login.html"),
        ("/style.css",   "style.css"),
        ("/tasks",       "tasks.html"),
        ("/assignments", "assignments.html"),
        ("/group",       "group.html"),
    ];
    let mut router = Router::new();
    for (path, real_path) in static_routes {
        let data: &'static str = String::leak(
            std::fs::read_to_string("frontend/static/".to_owned() + real_path)
                .unwrap_or_else(|_| panic!("missing file: {real_path}")),
        );
        router = router.route(path, get(move || async move { Html(data) }));
    }

    let json_data: &'static str = String::leak(
        std::fs::read_to_string("frontend/static/data.json")
            .expect("missing file: data.json"),
    );
    router = router.route(
        "/data.json",
        get(move || async move {
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/json")],
                json_data,
            )
        }),
    );

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Starting at http://0.0.0.0:3000");
    axum::serve(listener, router).await.unwrap();
}
