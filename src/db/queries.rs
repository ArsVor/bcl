use std::collections::HashSet;

use rusqlite::{Connection, Result, Transaction, params};

use crate::{cli::structs::Command, err_exit};

use super::models::{Bike, Category};

pub fn get_included_excluded(
    conn: &Connection,
    command: Command,
    table: &str,
) -> Result<(HashSet<i32>, HashSet<i32>)> {
    let mut include_id: HashSet<i32> = HashSet::new();
    let mut exclude_id: HashSet<i32> = HashSet::new();

    if !command.exclude_tags.is_empty() {
        for tag in command.exclude_tags {
            let id_set: HashSet<i32> = get_id_with_tag(conn, table, tag.as_str())?;
            exclude_id.extend(id_set);
        }
    }

    if !command.include_tags.is_empty() {
        for tag in command.include_tags {
            let id_set: HashSet<i32> = get_id_with_tag(conn, table, tag.as_str())?;
            include_id.extend(id_set);
        }
    }

    Ok((include_id, exclude_id))
}

pub fn get_category(conn: &Connection, abbr: &str) -> Result<Category> {
    conn.query_row(
        "SELECT * FROM category WHERE abbr = ?1",
        params![abbr],
        |row| Ok(Category::from_row(row)),
    )?
}

pub fn get_bike(conn: &Connection, abbr: &str, bike_id: u8) -> Result<Bike> {
    let mut stmt = conn.prepare(
        "SELECT * FROM bike b 
        JOIN category c ON c.id = b.category_id
        WHERE c.abbr = ?1",
    )?;
    let mut rows = stmt.query([abbr])?;

    for _ in 0..bike_id - 1 {
        _ = rows.next()?;
    }

    let row = rows.next()?;

    Bike::from_row(row.unwrap())
}

pub fn tag_get_or_create(conn: &Connection, tag_name: &str) -> Result<i32> {
    if let Ok(id) = conn.query_row(
        "SELECT id FROM tag WHERE name = ?1",
        params![tag_name],
        |row| row.get(0),
    ) {
        return Ok(id);
    }

    conn.execute("INSERT INTO tag (name) VALUES (?1)", params![tag_name])?;
    Ok(conn.last_insert_rowid() as i32)
}

pub fn tag_get_or_create_tx(tx: &Transaction, tag_name: &str) -> Result<i32> {
    if let Ok(id) = tx.query_row(
        "SELECT id FROM tag WHERE name = ?1",
        params![tag_name],
        |row| row.get(0),
    ) {
        return Ok(id);
    }

    tx.execute("INSERT INTO tag (name) VALUES (?1)", params![tag_name])?;
    Ok(tx.last_insert_rowid() as i32)
}
// pub fn tag_get_or_create(conn: &mut Connection, name: &str) -> Result<i32> {
//     match conn.query_row(
//         "SELECT id FROM tag WHERE name = ?1",
//         params![name],
//         |row| row.get(0),
//     ) {
//         Ok(id) => Ok(id),
//         Err(rusqlite::Error::QueryReturnedNoRows) => {
//             conn.execute(
//                 "INSERT INTO tag (name) VALUES (?1)",
//                 params![name],
//             )?;
//             Ok(conn.last_insert_rowid() as i32)
//         }
//         Err(e) => Err(e),
//     }
// }

pub fn get_id_with_tag(conn: &Connection, table: &str, name: &str) -> Result<HashSet<i32>> {
    let mut result: HashSet<i32> = HashSet::new();
    let select_sql: &str = match table {
        "buy" => {
            "SELECT b.id
             FROM buy b
             JOIN tag_to_buy tb ON b.id = tb.buy_id
             JOIN tag t ON tb.tag_id = t.id
             WHERE t.name = ?1"
        }
        "ride" => {
            "SELECT r.id
             FROM ride r
             JOIN tag_to_ride tr ON r.id = tr.ride_id
             JOIN tag t ON tr.tag_id = t.id
             WHERE t.name = ?1"
        }
        _ => {
            err_exit!("OOps!");
        }
    };

    let mut stmt = conn.prepare(select_sql)?;
    let buy_ids = stmt.query_map([name], |row| row.get::<_, i32>(0))?;

    for id in buy_ids {
        result.insert(id?);
    }

    Ok(result)
}

pub fn get_buy_id_with_cat(conn: &Connection, abbr: &str) -> Result<HashSet<i32>> {
    let mut result: HashSet<i32> = HashSet::new();
    let mut stmt = conn.prepare(
        "SELECT b.id
         FROM buy b
         JOIN buy_to_category bc ON b.id = bc.buy_id
         JOIN category c ON bc.category_id = c.id
         WHERE c.abbr = ?1",
    )?;
    let buy_ids = stmt.query_map([abbr], |row| row.get::<_, i32>(0))?;

    for id in buy_ids {
        result.insert(id?);
    }

    Ok(result)
}

pub fn get_buy_id_with_bike(conn: &Connection, bike_id: i32) -> Result<HashSet<i32>> {
    let mut result: HashSet<i32> = HashSet::new();
    let mut stmt = conn.prepare(
        "SELECT b.id
         FROM buy b
         JOIN buy_to_bike bbk ON b.id = bbk.buy_id
         JOIN bike bk ON bbk.bike_id = bk.id
         WHERE bk.id = ?1",
    )?;
    let buy_ids = stmt.query_map([bike_id], |row| row.get::<_, i32>(0))?;

    for id in buy_ids {
        result.insert(id?);
    }

    Ok(result)
}
