use std::collections::HashMap;

use chrono::NaiveDate;
use owo_colors::OwoColorize;
use rusqlite::{Connection, Result, params};

use crate::cli::structs::Command;
use crate::db::models::{Bike, Category};
use crate::db::queries::{get_bike, get_category, get_lub_info, tag_get_or_create_tx};
use crate::err_exit;

pub fn route(mut conn: Connection, command: Command) -> Result<()> {
    let obj = command.object.unwrap();
    match obj.as_str() {
        "bike" => bike(&conn, command),
        "buy" => buy(&mut conn, command),
        "cat" => category(&conn, command),
        "lub" => chain_lub(&conn, command),
        "ride" => ride(&mut conn, command),
        _ => Ok(()),
    }
}

fn category(conn: &Connection, command: Command) -> Result<()> {
    if command.category.is_none() || command.annotation.is_empty() {
        err_exit!(
            "Command params missed.\nExpected: `bcl add cat:[marker] [\"Category name\"] {OPT}`"
        );
    }

    let abbr: String = command.category.unwrap();
    let name: String = command.annotation.join(" ");

    conn.execute(
        "INSERT INTO category (abbr, name) VALUES (?1, ?2)",
        params![abbr, name],
    )?;

    println!(
        "{}",
        format!("Bicycle category: \"{0}\" ({1}) - added.", &abbr, &name).blue()
    );
    Ok(())
}

fn bike(conn: &Connection, command: Command) -> Result<()> {
    if command.category.is_none() || command.annotation.is_empty() {
        err_exit!(
            "Command params missed.\nExpected: `bcl add bike:[marker] [\"Bicycle name\"] {OPT}`"
        );
    }

    let date: NaiveDate = command.date.to_naive();
    let cat: Category = get_category(conn, command.category.unwrap().as_str()).unwrap();
    let name: String = command.annotation.join(" ");

    let mut id_in_cat: i32 = conn
        .query_row(
            "SELECT id_in_cat
        FROM bike
        WHERE category_id = ?1
        ORDER BY id DESC
        LIMIT 1",
            params![cat.id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    id_in_cat += 1;

    conn.execute(
        "INSERT INTO bike (category_id, id_in_cat, name, datestamp) VALUES (?1, ?2, ?3, ?4)",
        params![cat.id, id_in_cat, name, date],
    )?;

    println!(
        "{}",
        format!("{0} \"{1}\" - added", &cat.name, &name).blue()
    );
    Ok(())
}

fn chain_lub(conn: &Connection, command: Command) -> Result<()> {
    if command.category.is_none() || command.bike_id.is_none() {
        err_exit!("Command params missed.\nExpected: `bcl add lub [marker]:[bike_id] {OPT}`");
    }

    let date: NaiveDate = command.date.to_naive();
    let bike: Bike = get_bike(
        conn,
        command.category.unwrap().as_str(),
        command.bike_id.unwrap(),
    )
    .unwrap();
    let annotation: Option<String> = if command.annotation.is_empty() {
        None
    } else {
        Some(command.annotation.join(" "))
    };

    let distance_between_lubs: f32 = conn.query_row(
        "SELECT
            COALESCE(SUM(r.distance), 0.00)
        FROM ride r
        WHERE bike_id = ?1 AND datestamp <= ?2",
        params![bike.id, date], 
        |row| row.get(0)
    )?;

    conn.execute(
        "INSERT INTO chain_lubrication (bike_id, datestamp, distance, annotation) VALUES (?1, ?2, ?3, ?4)",
        params![bike.id, date, distance_between_lubs, annotation],
    )?;

    println!(
        "{}",
        format!(
            "Chain lubrication from {} - added.",
            &date.format("%d.%m.%y").to_string()
        )
        .blue()
    );

    Ok(())
}

fn buy(conn: &mut Connection, command: Command) -> Result<()> {
    if command.val.is_none() || command.annotation.is_empty() {
        err_exit!(
            "Command params missed.\nExpected: `bcl add buy [\"Product name\"] [price] {OPT}`"
        );
    }

    let name: String = command.annotation.join(" ");
    let price: f32 = command.val.unwrap();
    let date: NaiveDate = command.date.to_naive();

    let tx = conn.transaction()?; // все піде сюди

    // 1. Теги в тій же транзакції
    let mut tags_id: Vec<i32> = Vec::new();
    if !command.include_tags.is_empty() {
        for tag in &command.include_tags {
            tags_id.push(tag_get_or_create_tx(&tx, tag.as_str())?);
        }
    }

    // 2. Категорія і байк
    let mut fk_id: HashMap<&str, i32> = HashMap::new();
    if let Some(cat_name) = &command.category.get() {
        let cat: Category = get_category(&tx, cat_name.as_str())?;
        fk_id.insert("cat_id", cat.id);

        if let Some(bike_id_val) = command.bike_id.get() {
            let bike: Bike = get_bike(&tx, cat_name.as_str(), bike_id_val)?;
            fk_id.insert("bike_id", bike.id);
        }
    }

    // 3. Основна вставка покупки
    tx.execute(
        "INSERT INTO buy (name, price, datestamp) VALUES (?1, ?2, ?3)",
        params![name, price, date],
    )?;
    let buy_id = tx.last_insert_rowid();

    // 4. Прив’язки
    if let Some(&category_id) = fk_id.get("cat_id") {
        tx.execute(
            "INSERT INTO buy_to_category (buy_id, category_id) VALUES (?1, ?2)",
            params![buy_id, category_id],
        )?;
    }

    if let Some(&bike_id) = fk_id.get("bike_id") {
        tx.execute(
            "INSERT INTO buy_to_bike (buy_id, bike_id) VALUES (?1, ?2)",
            params![buy_id, bike_id],
        )?;
    }

    for tag_id in tags_id {
        tx.execute(
            "INSERT INTO tag_to_buy (tag_id, buy_id) VALUES (?1, ?2)",
            params![tag_id, buy_id],
        )?;
    }

    // 5. Комміт
    tx.commit()?;

    println!(
        "{}",
        format!(
            "Purchase: \"{0}\" from {1} - added.",
            &name,
            &date.format("%d.%m.%y")
        )
        .blue()
    );

    Ok(())
}

fn ride(conn: &mut Connection, command: Command) -> Result<()> {
    if command.category.is_none() || command.bike_id.is_none() || command.val.is_none() {
        err_exit!(
            "Command params missed.\nExpected: `bcl add ride [category]:[bike_id] [distance] {OPT}`"
        );
    }

    let date: NaiveDate = command.date.to_naive();
    let bike: Bike = get_bike(
        conn,
        command.category.unwrap().as_str(),
        command.bike_id.unwrap(),
    )
    .unwrap();
    let distance: f32 = command.val.unwrap();
    let annotation: Option<String> = if command.annotation.is_empty() {
        None
    } else {
        Some(command.annotation.join(" "))
    };

    let after_lub_distance: f32 = if let Ok(prev_distance) = get_lub_info(conn, bike.id as u8) {
        prev_distance + distance
    } else {
        err_exit!("Can't calculate distance after last lubrication.");
    };

    let tx = conn.transaction()?;

    let mut tags_id: Vec<i32> = Vec::new();
    if !command.include_tags.is_empty() {
        for tag in &command.include_tags {
            tags_id.push(tag_get_or_create_tx(&tx, tag.as_str())?);
        }
    }

    tx.execute(
        "INSERT INTO ride (bike_id, datestamp, distance, annotation) VALUES (?1, ?2, ?3, ?4)",
        params![bike.id, date, distance, annotation],
    )?;

    let ride_id: i32 = tx.last_insert_rowid() as i32;

    for tag_id in tags_id {
        tx.execute(
            "INSERT INTO tag_to_ride (tag_id, ride_id) VALUES (?1, ?2)",
            params![tag_id, ride_id],
        )?;
    }

    tx.commit()?;

    let msg = format!(
        "After last chain lubrication you ride: {}km",
        &after_lub_distance
    );

    println!(
        "{}",
        format!(
            "Ride {0}:{1} to {2}km from {3} - added",
            &command.category.unwrap(),
            &command.bike_id.unwrap(),
            &distance,
            &date.format("%d.%m.%y").to_string()
        )
        .blue()
    );

    if after_lub_distance > 200.00 {
        println!("{}", msg.red());
    } else if after_lub_distance > 150.00 {
        println!("{}", msg.yellow());
    } else {
        println!("{}", msg.green());
    };

    Ok(())
}
