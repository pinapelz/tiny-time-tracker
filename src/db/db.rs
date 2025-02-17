use chrono::Local;
use rusqlite::{Connection, Result};

pub fn create_db() -> Result<()> {
    let conn = Connection::open("ttt.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name VARCHAR(255),
            diskpath TEXT UNIQUE NOT NULL,
            filepath TEXT UNIQUE NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS records (
            id INTEGER PRIMARY KEY,
            active_time INTEGER NOT NULL,
            FOREIGN KEY(id) REFERENCES tasks(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS active (
            id INTEGER PRIMARY KEY,
            datetime DATETIME NOT NULL,
            FOREIGN KEY(id) REFERENCES tasks(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
        session_id INTEGER PRIMARY KEY AUTOINCREMENT,
        id INTEGER NOT NULL,
        start_time DATETIME NOT NULL,
        end_time DATETIME NOT NULL,
        FOREIGN KEY(id) REFERENCES tasks(id)
    )",
        [],
    )?;
    Ok(())
}

pub fn insert_task(id: &str, name: &str, diskpath: &str, filepath: &str) -> Result<()> {
    let conn = Connection::open("ttt.db")?;
    conn.execute(
        "INSERT INTO tasks (id, name, diskpath, filepath) VALUES (?1, ?2, ?3, ?4)",
        &[id, name, diskpath, filepath],
    )?;
    print!("Inserted task with id: {}", id);
    Ok(())
}

pub fn get_next_id() -> Result<i64> {
    let conn = Connection::open("ttt.db")?;
    let mut stmt = conn.prepare("SELECT COALESCE(MAX(id), 0) + 1 FROM tasks")?;
    let next_id: i64 = stmt.query_row([], |row| row.get(0))?;
    Ok(next_id)
}

fn get_active_task_start_time(id: &str) -> Result<String> {
    let conn = Connection::open("ttt.db")?;
    let mut stmt = conn.prepare("SELECT datetime FROM active WHERE id = ?1")?;
    let start_time: String = stmt.query_row(&[id], |row| row.get(0))?;
    Ok(start_time)
}

pub fn file_path_already_tracked(filepath: &str) -> Result<bool> {
    let conn = Connection::open("ttt.db")?;
    let mut stmt = conn.prepare("SELECT EXISTS(SELECT 1 FROM tasks WHERE filepath = ?1)")?;
    let exists: i64 = stmt.query_row(&[filepath], |row| row.get(0))?;
    Ok(exists == 1)
}
