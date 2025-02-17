use rusqlite::{Connection, Result};
use serde::Serialize;



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

pub fn get_all_tasks() -> Result<Vec<(i64, String, String, i64)>> {
    let conn = Connection::open("ttt.db")?;
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, 
        CASE 
            WHEN a.id IS NOT NULL THEN 'Running' 
            ELSE COALESCE(
                (SELECT MAX(s.start_time) FROM sessions s WHERE s.id = t.id), 
                'Never'
            ) 
        END AS last_opened,
        COALESCE((SELECT SUM(r.active_time) FROM records r WHERE r.id = t.id), 0) AS total_playtime
        FROM tasks t
        LEFT JOIN active a ON t.id = a.id"
    )?;
    let tasks = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))?
        .collect::<Result<Vec<_>>>()?;
    Ok(tasks)
}
