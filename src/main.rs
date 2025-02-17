// mod scheduler;

// fn main() {
//     let xml_path = "C:\\Users\\donal\\Repositories\\tiny-time-tracker\\NotepadTrigger.xml"; // Change this to the actual path
//     let task_name = "TestTaskTemplate";

//     if let Err(e) = scheduler::create_scheduled_task(xml_path, task_name) {
//         eprintln!("Error creating task: {}", e);
//     }
//     db::db::create_db().expect("Error creating database");
// }

use axum::{
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
mod mswin;
mod scheduler;
mod db;

#[tokio::main]
async fn main() {
    start_web_server().await;
}

async fn start_web_server() {
    db::db::create_db().expect("Error creating database");
    let app = Router::new()
        .route("/", get(index))
        .route("/create", post(create_new_tracked_app))
        .nest_service("/static", ServeDir::new("static"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Serving on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn create_new_tracked_app() -> impl IntoResponse {
    let filepath = mswin::filechooser_select_executable();
    let volume_path = mswin::get_device_path(&filepath).unwrap();
    if let Err(e) = scheduler::create_scheduled_task(
        "Template2Generated4",
        &volume_path,
    ) {
        eprintln!("Error creating scheduled task: {}", e);
    }
    Html(format!(
        "Filepath: {}, Volume Path: {}",
        filepath, volume_path.to_string()
    ))
}