use super::schema::init_schema;
use rusqlite::{Connection, Result};

pub fn open_connection_with_fk(path: &str) -> Result<Connection, rusqlite::Error> {
    let db_exist: bool = std::path::Path::new(path).exists();

    let conn = Connection::open(path)?;
    conn.execute("PRAGMA foreign_keys = ON;", [])?;

    if !db_exist {
        init_schema(&conn)?;
    }
    Ok(conn)
}
