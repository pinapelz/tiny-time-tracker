use axum::{
    extract::Path,
    extract::Query,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
    Form
};
use mswin::show_confirmation_dialog;
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
struct ModificationForm {
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
    enable_auto_cleanup_active_task();
    let app = Router::new()
        .route("/", get(index))
        .route("/create", post(create_new_tracked_app))
        .route("/modify_path", post(modify_app_path))
        .route("/delete", post(delete_tracked_app))
        .route("/tasks", get(get_tasks))
        .route("/task/:id", get(task_detailed_view_page))
        .route("/disablecleanup", get(disable_auto_cleanup_active_task))
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
    let filepath = match mswin::filechooser_select_executable() {
        Some(path) => path,
        None => return Html("File selection canceled.".to_string()),
    };


    let volume_path = mswin::get_device_path(&filepath)
        .unwrap_or_else(|err| {
            eprintln!("Error getting device path: {}", err);
            String::new()
        });

    if volume_path == String::new(){
        return Html("Failed to convert to volume path.".to_string())
    }


    if db::db::file_path_already_tracked(db_path.as_str(), filepath.as_str()).unwrap(){
        if !show_confirmation_dialog(
        "File Already Exists",
        "This executable is already tracked or was previously tracked. Do you want to add it anyways?"){
            return Html(format!(
                "Task creation cancelled. No change has occured",
            ))
        }
    }

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

async fn delete_tracked_app(Form(form): Form<ModificationForm>) -> impl IntoResponse {
    dotenv().ok();
    let db_path = env::var("DB_PATH")
        .expect("DB_PATH must be set in .env file");

    db::db::disable_task(&db_path, form.id).unwrap();
    scheduler::delete_scheduled_task(&form.id.to_string()).unwrap();
    Html(format!("Program with ID: {} has been deleted. You can still navigate to this URL to view details later", form.id))
}

async fn modify_app_path(Form(form): Form<ModificationForm>) -> impl IntoResponse {
    dotenv().ok();
    // easier to just delete and then re-create the task
    let db_path = env::var("DB_PATH").expect("DB_PATH must be in .env file");
    let trigger_exe_path = env::var("TRIGGER_EXE_PATH")
    .expect("TRIGGER_EXE_PATH must be set in .env file");

    let filepath = match mswin::filechooser_select_executable() {
        Some(path) => path,
        None => return Html("File selection canceled.".to_string()),
    };
    scheduler::delete_scheduled_task(&form.id.to_string()).unwrap();
    let launch_task_name = format!("OnLaunchTinyTimeTracker{}", form.id.to_string());
    let close_task_name = format!("OnCloseTinyTimeTracker{}", form.id.to_string());

    let volume_path = mswin::get_device_path(&filepath)
        .unwrap_or_else(|err| {
            eprintln!("Error getting device path: {}", err);
            String::new()
        });

    if volume_path == String::new(){
        return Html("Failed to convert to volume path.".to_string())
    }
    
    if let Err(e) = scheduler::create_scheduled_task(
        &launch_task_name,
        &volume_path,
        "4688",
        &trigger_exe_path,
        &form.id.to_string(),
        &db_path,
    ) {
        eprintln!("Error creating scheduled task for Launch: {}", e);
    }
    if let Err(e) = scheduler::create_scheduled_task(
        &close_task_name,
        &volume_path,
        "4689",
        &trigger_exe_path,
        &form.id.to_string(),
        &db_path,
    ) {
        eprintln!("Error creating scheduled task for Launch: {}", e);
    }
    if let Err(e) = db::db::set_new_filepath(db_path.as_str(), form.id, &filepath) {
        eprintln!("Error updating file path: {}", e);
        return Html(format!("Failed to update file path: {}", e));
    }
    if let Err(e) = db::db::set_new_volumepath(db_path.as_str(), form.id, &volume_path) {
        eprintln!("Error updating volume path: {}", e);
        return Html(format!("Failed to update volume path: {}", e));
    }
    Html(format!("Executable path sucessfully changed! Refresh to see the result"))
}

fn enable_auto_cleanup_active_task(){
    dotenv().ok();
    let auto_cleanup = env::var("AUTO_CLEANUP_ACTIVE_TASKS")
    .map(|v| v.to_lowercase() == "true")
    .unwrap_or(false);
    if auto_cleanup{
        let trigger_exe_path = env::var("TRIGGER_EXE_PATH")
        .expect("TRIGGER_EXE_PATH must be set in .env file");
    let db_path = env::var("DB_PATH")
        .expect("DB_PATH must be set in .env file");
        scheduler::create_cleanup_active_task(db_path.as_str(), trigger_exe_path.as_str()).unwrap()
    }
}

// shouldn't normally be used since you never want a task to be stuck in active table
// however, its there if for some reason you want a clean way to delete it
async fn disable_auto_cleanup_active_task() -> impl IntoResponse{
    scheduler::delete_cleanup_task().unwrap();
    Html(format!("Cleaning up active tasks on login is disabled. Task deleted, if you'd like this to persist please change the relevant environment variable to false"))
}
