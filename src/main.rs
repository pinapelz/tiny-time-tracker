use axum::{
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
    Form
};
use serde::Deserialize;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
mod mswin;
mod scheduler;
mod db;
use dotenv::dotenv;
use std::env;

#[derive(Deserialize)]
struct CreateTaskForm {
    task_name: String,
}

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

async fn create_new_tracked_app(Form(form): Form<CreateTaskForm>) -> impl IntoResponse {
    dotenv().ok();
    let trigger_exe_path = env::var("TRIGGER_EXE_PATH")
        .expect("TRIGGER_EXE_PATH must be set in .env file");
    let db_path = env::var("DB_PATH")
        .expect("DB_PATH must be set in .env file");

    let next_available_id = db::db::get_next_id().unwrap();
    let filepath = mswin::filechooser_select_executable();
    let volume_path = mswin::get_device_path(&filepath).unwrap();
    let launch_task_name = format!("OnLaunchTinyTimeTracker{}", next_available_id);
    let close_task_name = format!("OnCloseTinyTimeTracker{}", next_available_id);

    if let Err(e) = scheduler::create_scheduled_task(
        &launch_task_name,
        &volume_path,
        "4688",
        &trigger_exe_path,
        &next_available_id.to_string(),
        &db_path,
    ) {
        eprintln!("Error creating scheduled task for Launch: {}", e);
    }
    if let Err(e) = scheduler::create_scheduled_task(
        &close_task_name,
        &volume_path,
        "4689",
        &trigger_exe_path,
        &next_available_id.to_string(),
        &db_path,
    ) {
        eprintln!("Error creating scheduled task for Launch: {}", e);
    }

    db::db::insert_task(
        &next_available_id.to_string(),
        &form.task_name,
        &volume_path,
        &filepath,
    );
    Html(format!(
        "Application '{}' tracked successfully.<br>Filepath: {}<br>Volume Path: {}",
        form.task_name,
        filepath,
        volume_path.to_string()
    ))
}