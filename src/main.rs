use axum::{
    extract::Path,
    extract::Query,
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
use serde::Deserialize;
use askama::Template;


#[derive(Deserialize)]
struct CreateTaskForm {
    task_name: String,
    notes: Option<String>,
    #[serde(default)]
    create_scheduled_tasks: bool,
}

#[derive(Deserialize)]
struct DeletionForm {
    id: i64,
}

#[tokio::main]
async fn main() {
    start_web_server().await;
}

#[derive(Template, Debug)]
#[template(path = "task_detail.html")]
struct TaskDetailTemplate {
    id: i64,
    name: String,
    last_opened: String,
    total_playtime: String,
    notes: String,
    session_count: i64,
    filepath: String,
    volume_path: String,
    sessions: Vec<(String, String)>,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    show_disabled: bool
}


#[derive(Deserialize)]
struct TasksParams {
    show_disabled: Option<bool>,
}


async fn start_web_server() {
    db::db::create_db().expect("Error creating database");
    let app = Router::new()
        .route("/", get(index))
        .route("/create", post(create_new_tracked_app))
        .route("/delete", post(delete_tracked_app))
        .route("/tasks", get(get_tasks))
        .route("/task/:id", get(task_detailed_view_page))
        .nest_service("/static", ServeDir::new("static"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Serving on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index(Query(params): Query<TasksParams>) -> impl IntoResponse {
    let template = IndexTemplate {
        show_disabled: params.show_disabled.unwrap_or(false)
    };
    
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template error: {}", e)
        ).into_response(),
    }
}

async fn get_tasks(Query(params): Query<TasksParams>) -> impl IntoResponse {
    dotenv().ok();
    let db_path = env::var("DB_PATH")
        .expect("DB_PATH must be set in .env file");
    
    match db::db::get_all_tasks(&db_path, params.show_disabled.unwrap_or(false)) {
        Ok(tasks) => Json(tasks).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch tasks: {}", e)
        ).into_response(),
    }
}


async fn task_detailed_view_page(Path(id): Path<String>) -> impl IntoResponse {
    dotenv().ok();
    let db_path = env::var("DB_PATH")
        .expect("DB_PATH must be set in .env file");

    match db::db::get_task_by_id(&db_path, &id) {
        Ok((id, name, last_opened, total_playtime, notes, session_count, filepath, volume_path, sessions)) => {
            let total_playtime = {
                let duration = chrono::Duration::seconds(total_playtime);
                let hours = duration.num_hours();
                let minutes = duration.num_minutes() % 60;
                let seconds = duration.num_seconds() % 60;
                format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
            };
            let template = TaskDetailTemplate {
                id,
                name,
                last_opened,
                total_playtime,
                notes,
                session_count,
                filepath,
                volume_path,
                sessions,
            };
            match template.render() {
                Ok(html) => Html(html).into_response(),
                Err(e) => (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Template error: {}", e)
                ).into_response(),
            }
        },
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch task: {}", e)
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

    if form.create_scheduled_tasks {
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
    dotenv().ok();
    let db_path = env::var("DB_PATH")
        .expect("DB_PATH must be set in .env file");

    db::db::disable_task(&db_path, form.id).unwrap();
    scheduler::delete_scheduled_task(&form.id.to_string()).unwrap();
    Html(format!("Program with ID: {} has been deleted. You can still navigate to this URL to view details later", form.id))
}