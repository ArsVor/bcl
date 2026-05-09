use anyhow::Result;
use std::path::Path;

use super::schema::init_schema;
use rusqlite::Connection;

pub fn open_connection_with_fk(path: &Path) -> Result<Connection> {
    let db_exist: bool = path.exists();

    let conn = Connection::open(path)?;
    conn.execute("PRAGMA foreign_keys = ON;", [])?;

    if !db_exist {
        init_schema(&conn)?;
    }
    Ok(conn)
}
