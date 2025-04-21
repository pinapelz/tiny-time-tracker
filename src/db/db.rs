use rusqlite::{Connection, Result};

pub fn create_db() -> Result<()> {
    let conn = Connection::open("ttt.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            notes TEXT NULL,
            name VARCHAR(255),
            diskpath TEXT NOT NULL,
            filepath TEXT NOT NULL,
            enabled BOOLEAN NOT NULL DEFAULT 1
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

pub fn insert_task(
    db_path: &str,
    id: &str,
    name: &str,
    notes: &str,
    diskpath: &str,
    filepath: &str,
) -> Result<()> {
    let conn = Connection::open(db_path)?;
    conn.execute(
        "INSERT INTO tasks (id, name, notes, diskpath, filepath) VALUES (?1, ?2, ?3, ?4, ?5)",
        &[id, name, notes, diskpath, filepath],
    )?;
    print!("Inserted task with id: {}", id);
    Ok(())
}

pub fn get_next_id(db_path: &str) -> Result<i64> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("SELECT COALESCE(MAX(id), 0) + 1 FROM tasks")?;
    let next_id: i64 = stmt.query_row([], |row| row.get(0))?;
    Ok(next_id)
}

pub fn file_path_already_tracked(db_path: &str, filepath: &str) -> Result<bool> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("SELECT EXISTS(SELECT 1 FROM tasks WHERE filepath = ?1)")?;
    let exists: i64 = stmt.query_row(&[filepath], |row| row.get(0))?;
    Ok(exists == 1)
}

pub fn disable_task(db_path: &str, id: i64) -> Result<bool> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("UPDATE tasks SET enabled = 0 WHERE id = ?1")?;
    let success = stmt.execute([id])?;
    assert!(success == 1);
    Ok(true)
}

pub fn set_new_filepath(db_path: &str, id: i64, new_filepath: &str) -> Result<bool> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("UPDATE tasks SET filepath = ?1 WHERE id = ?2")?;
    stmt.execute([new_filepath, &id.to_string()])?;
    Ok(true)
}

pub fn set_new_volumepath(db_path: &str, id: i64, new_volumepath: &str) -> Result<bool> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("UPDATE tasks SET diskpath = ?1 WHERE id = ?2")?;
    stmt.execute([new_volumepath, &id.to_string()])?;
    Ok(true)
}

pub fn get_activity_year(db_path: &str, id: i64) -> Result<Vec<(String, i64)>> {
    let conn = Connection::open(db_path)?;
    if id != -1 {
        let mut stmt = conn.prepare(
            "SELECT DATE(start_time) AS session_date, COUNT(*) AS session_count
            FROM sessions
            WHERE DATE(start_time) >= DATE('now', '-365 days')
            AND id = ?1
            GROUP BY session_date
            ORDER BY session_date ASC;",
        )?;
        let activity = stmt
            .query_map([id], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>>>()?;
        Ok(activity)
    } else {
        let mut stmt = conn.prepare(
            "SELECT DATE(start_time) AS session_date, COUNT(*) AS session_count
            FROM sessions
            WHERE DATE(start_time) >= DATE('now', '-365 days')
            GROUP BY session_date
            ORDER BY session_date ASC;",
        )?;
        let activity = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>>>()?;
        Ok(activity)
    }
}

// id, name, last_opened, total_playtime, notes, session_count, filepath, volume_path, sessions
pub fn get_task_by_id(
    db_path: &str,
    id: &str,
) -> Result<(
    i64,
    String,
    String,
    i64,
    String,
    i64,
    String,
    String,
    Vec<(String, String)>,
)> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name,
        CASE
            WHEN a.id IS NOT NULL THEN 'Running'
            ELSE COALESCE(
                (SELECT MAX(s.start_time) FROM sessions s WHERE s.id = t.id),
                'Never'
            )
        END AS last_opened,
        CASE
            WHEN a.id IS NOT NULL THEN
                COALESCE((SELECT SUM(r.active_time) FROM records r WHERE r.id = t.id), 0) +
                (strftime('%s', 'now') - strftime('%s', a.datetime))
            ELSE
                COALESCE((SELECT SUM(r.active_time) FROM records r WHERE r.id = t.id), 0)
        END AS total_playtime,
        COALESCE(t.notes, '') as notes,
        COALESCE((SELECT COUNT(*) FROM sessions s WHERE s.id = t.id), 0) as session_count,
        t.filepath,
        t.diskpath
        FROM tasks t
        LEFT JOIN active a ON t.id = a.id
        WHERE t.id = ?1",
    )?;
    let task = stmt.query_row(&[id], |row| {
        Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
            row.get(6)?,
            row.get(7)?,
        ))
    })?;

    let mut session_stmt =
        conn.prepare("SELECT start_time, end_time FROM sessions WHERE id = ?1")?;
    let sessions = session_stmt
        .query_map(&[id], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<Vec<_>>>()?;

    Ok((
        task.0, task.1, task.2, task.3, task.4, task.5, task.6, task.7, sessions,
    ))
}

pub fn get_all_tasks(
    db_path: &str,
    include_disabled: bool,
) -> Result<Vec<(i64, String, String, i64, String, i64)>> {
    let conn = Connection::open(db_path)?;
    let where_clause = if !include_disabled {
        "WHERE t.enabled = 1"
    } else {
        ""
    };
    let query = format!(
        "SELECT t.id, t.name,
        CASE
            WHEN a.id IS NOT NULL THEN 'Running'
            ELSE COALESCE(
                (SELECT MAX(s.start_time) FROM sessions s WHERE s.id = t.id),
                'Never'
            )
        END AS last_opened,
        COALESCE((SELECT SUM(r.active_time) FROM records r WHERE r.id = t.id), 0) AS total_playtime,
        COALESCE(t.notes, '') as notes,
        COALESCE((SELECT COUNT(*) FROM sessions s WHERE s.id = t.id), 0) as session_count
        FROM tasks t
        LEFT JOIN active a ON t.id = a.id
        {}",
        where_clause
    );

    let mut stmt = conn.prepare(&query)?;
    let tasks = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(tasks)
}
