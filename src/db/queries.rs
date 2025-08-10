use rusqlite::{params, Connection, Result};
use rusqlite_from_row::FromRow;

use super::models::{Bike, Category};


pub fn get_category(conn: &Connection, abbr: &str) -> Result<Category> {
    conn.query_row(
        "SELECT * FROM category WHERE abbr = ?1",
        params![abbr],
        |row| {
            Ok (Category::try_from_row(row))
        },
    )?
}

pub fn get_bike(conn: &Connection, category_id: i32, bike_id: u8) -> Result<Bike> {
    let mut stmt = conn.prepare("SELECT * FROM bike where category_id = ?1")?;
    let mut rows = stmt.query([category_id])?;

    for _ in 0..bike_id {
        _ = rows.next()?;
    }

    let row = rows.next()?;

    Bike::try_from_row(row.unwrap())
}

pub fn tag_get_or_create(conn: &mut Connection, name: &str) -> Result<i32> {
    match conn.query_row(
        "SELECT id FROM tag WHERE name = ?1",
        params![name],
        |row| row.get(0),
    ) {
        Ok(id) => Ok(id),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            conn.execute(
                "INSERT INTO tag (name) VALUES (?1)",
                params![name],
            )?;
            Ok(conn.last_insert_rowid() as i32)
        }
        Err(e) => Err(e),
    }
}
