use axum::{
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
    Form
};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
mod mswin;
mod scheduler;
mod db;
use dotenv::dotenv;
use std::env;
use serde::{Deserialize, Serialize};


#[derive(Deserialize)]
struct CreateTaskForm {
    task_name: String,
    notes: Option<String>,
}

#[derive(Deserialize)]
struct DeletionForm {
    id: i64,
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
        .route("/delete", post(delete_tracked_app))
        .route("/tasks", get(get_tasks))
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

async fn get_tasks() -> impl IntoResponse {
    dotenv().ok();
    let db_path = env::var("DB_PATH")
    .expect("DB_PATH must be set in .env file");
    match db::db::get_all_tasks(&db_path) {
        Ok(tasks) => Json(tasks).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch tasks: {}", e)
        ).into_response(),
    }
}

async fn create_new_tracked_app(Form(form): Form<CreateTaskForm>) -> impl IntoResponse {
    dotenv().ok();
    let trigger_exe_path = env::var("TRIGGER_EXE_PATH")
        .expect("TRIGGER_EXE_PATH must be set in .env file");
    let db_path = env::var("DB_PATH")
        .expect("DB_PATH must be set in .env file");

    let next_available_id = db::db::get_next_id(&db_path).unwrap();
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
    if let Err(e) = db::db::insert_task(
        &db_path,
        &next_available_id.to_string(),
        &form.task_name,
        &form.notes.unwrap_or("".to_string()),
        &volume_path,
        &filepath,
    ) {
        eprintln!("Error inserting task into database: {}", e);
    }
    Html(format!(
        "Application '{}' tracked successfully.<br>Filepath: {}<br>Volume Path: {}",
        form.task_name,
        filepath,
        volume_path.to_string()
    ))
}

async fn delete_tracked_app(Form(form): Form<DeletionForm>) -> impl IntoResponse {
    // stub for deletion TODO
}