use rusqlite::{Connection, Result};
use chrono::Local;
use std::env;


fn get_active_task_start_time(db_path: &str, id: &str) -> Result<String> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("SELECT datetime FROM active WHERE id = ?1")?;
    let start_time: String = stmt.query_row(&[id], |row| row.get(0))?;
    Ok(start_time)
}

fn insert_session(db_path: &str, id: &str, start_time: &str, end_time: &str) -> Result<()> {
    let conn = Connection::open(db_path)?;
    conn.execute(
        "INSERT INTO sessions (id, start_time, end_time) VALUES (?1, ?2, ?3)",
        &[id, start_time, end_time],
    )?;
    Ok(())
}

fn increment_recorded_time(db_path: &str, id: &str, active_time: &str) -> Result<()> {
    let conn = Connection::open(db_path)?;
    conn.execute(
        "INSERT INTO records (id, active_time) VALUES (?1, ?2)
        ON CONFLICT(id) DO UPDATE SET active_time = active_time + ?2",
        &[id, active_time],
    )?;
    Ok(())
}

fn start_tracking(db_path: &str, id: &str) -> Result<()> {
    let conn = Connection::open(db_path)?;
    let curr_datetime = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO active (id, datetime) VALUES (?1, ?2)",
        &[id, &curr_datetime],
    )?;
    Ok(())
}

fn end_tracking(db_path: &str, id: &str) -> Result<()> {
    let conn = Connection::open(db_path)?;
    let curr_datetime = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let start_time = get_active_task_start_time(db_path, id)?;
    let start_time_parsed = chrono::NaiveDateTime::parse_from_str(&start_time, "%Y-%m-%d %H:%M:%S")
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    let curr_datetime_parsed = chrono::NaiveDateTime::parse_from_str(&curr_datetime, "%Y-%m-%d %H:%M:%S")
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    let duration = curr_datetime_parsed - start_time_parsed;
    let active_time = duration.num_seconds().to_string();
    insert_session(db_path, id, &start_time, &curr_datetime)?;
    increment_recorded_time(db_path, id, &active_time)?;
    conn.execute("DELETE FROM active WHERE id = ?1", &[id])?;
    Ok(())
}


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        eprintln!("Usage: {} <start|end> --id <id> --db <path>", args[0]);
        return;
    }

    let command = &args[1];
    let mut id = "";
    let mut db_path = "";

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--id" => {
                if i + 1 < args.len() {
                    id = &args[i + 1];
                    i += 2;
                } else {
                    eprintln!("Missing value for --id");
                    return;
                }
            }
            "--db" => {
                if i + 1 < args.len() {
                    db_path = &args[i + 1];
                    i += 2;
                } else {
                    eprintln!("Missing value for --db");
                    return;
                }
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                return;
            }
        }
    }

    if id.is_empty() || db_path.is_empty() {
        eprintln!("Usage: {} <start|end> --id <id> --db <path>", args[0]);
        return;
    }

    match command.as_str() {
        "start" => {
            start_tracking(db_path, id).expect("Error starting tracking");
        }
        "end" => {
            end_tracking(db_path, id).expect("Error ending tracking");
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Usage: {} <start|end> --id <id> --db <path>", args[0]);
        }
    }
}