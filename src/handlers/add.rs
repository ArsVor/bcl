use std::collections::HashMap;

use chrono::NaiveDate;
use rusqlite::{params, Connection, Result};

use crate::cli::structs::Command;
use crate::db::queries::{get_bike, get_category, tag_get_or_create};
use crate::db::models::{Bike, Category};
use crate::err_exit;

pub fn route(mut conn: Connection, command: Command) -> Result<()> {
    let obj = command.object.unwrap();
    match obj.as_str() {
        "bike" => bike(&conn, command),
        "buy" => buy(&mut conn, command),
        "cat" => category(&conn, command),
        "lub" => chain_lub(&conn, command),
        "ride" => ride(&mut conn, command),
        _ => Ok(())
    }
}

fn category(conn: &Connection, command: Command) -> Result<()> {
    if command.category.is_none() || command.annotation.is_empty() {
        err_exit!("Command params missed.\nExpected: `bcl add cat:[marker] [\"Category name\"]`");
    }

    let abbr: String = command.category.unwrap();
    let name: String = command.annotation.join(" ");

    conn.execute(
        "INSERT INTO category (abbr, name) VALUES (?1, ?2)",
        params![abbr, name],
    )?;
    
    println!("Bicycle category: \"{0}\" ({1}) - added.", &abbr, &name);
    Ok(())
}

fn bike(conn: &Connection, command: Command) -> Result<()> {
    if command.category.is_none() || command.annotation.is_empty() {
        err_exit!("Command params missed.\nExpected: `bcl add bike:[marker] [\"Bicycle name\"]`");
    }

    let date: NaiveDate = command.date.to_naive();
    let cat: Category = get_category(conn, command.category.unwrap().as_str()).unwrap();
    let name: String = command.annotation.join(" ");

    conn.execute(
        "INSERT INTO bike (category_id, name, datestamp) VALUES (?1, ?2, ?3)",
        params![cat.id, name, date]
    )?;

    println!("{0} \"{1}\" - added", &cat.name, &name);

    Ok(())
}

fn chain_lub(conn: &Connection, command: Command) -> Result<()> {
    if command.category.is_none() || command.bike_id.is_none() {
        err_exit!("Command params missed.\nExpected: `bcl add lub [marker]:[bike_id]`");
    }

    let date: NaiveDate = command.date.to_naive();
    let cat: Category = get_category(conn, command.category.unwrap().as_str()).unwrap();
    let bike: Bike = get_bike(conn, cat.id, command.bike_id.unwrap()).unwrap();
    let annotation: Option<String> = if command.annotation.is_empty() {
        None
    } else {
        Some(command.annotation.join(" "))
    };

    conn.execute(
        "INSERT INTO bike (bike_id, datestamp, annotation) VALUES (?1, ?2, ?3)",
        params![bike.id, date, annotation]
    )?;

    println!("Chain lubrication from {} - added.", &date.format("%d.%m.%y").to_string());

    Ok(())
}

fn buy(conn: &mut Connection, command: Command) -> Result<()> {
    if command.val.is_none() || command.annotation.is_empty() {
        err_exit!("Command params missed.\nExpected: `bcl add buy [\"Product name\"] [price]`");
    }

    let name: String = command.annotation.join(" ");
    let price: f32 = command.val.unwrap();
    let date: NaiveDate = command.date.to_naive();

    let mut tags_id: Vec<i32> = Vec::new();
    let mut fk_id: HashMap<&str, i32> = HashMap::new();

    if !command.include_tags.is_empty() {
        for tag in command.include_tags {
            tags_id.push(tag_get_or_create(conn, tag.as_str()).unwrap());
        }
    }
    
    if command.category.is_some() {
        let cat: Category = get_category(conn, command.category.unwrap().as_str()).unwrap();
        fk_id.insert("cat_id", cat.id);

        if command.bike_id.is_some() {
            let bike: Bike = get_bike(conn, cat.id, command.bike_id.unwrap()).unwrap();
            fk_id.insert("bike_id", bike.id);
        }
    }

    let tx = conn.transaction()?;

    tx.execute(
        "INSERT INTO buy (name, price, datestamp) VALUES (?1, ?2, &3)",
        params![name, price, date],
    )?;    

    let buy_id = tx.last_insert_rowid();

    if fk_id.contains_key(&"cat_id") {
        let category_id = fk_id.get("cat_id");
        tx.execute(
            "INSERT INTO buy_to_category (buy_id, category_id) VALUES (?1, ?2)",
            params![buy_id, category_id]
        )?;
    }

    if fk_id.contains_key(&"bike_id") {
        let bike_id = fk_id.get("bike_id");
        tx.execute(
            "INSERT INTO buy_to_bike (buy_id, bike_id) VALUES (?1, ?2)",
            params![buy_id, bike_id]
        )?;
    }

    for tag_id in tags_id {
        tx.execute(
            "INSERT INTO tag_to_buy (tag_id, buy_id) VALUES (?1, ?2)",
            params![tag_id, buy_id]
        )?;
    }

    tx.commit()?;

    println!("Purchase: \"{0}\" from {1} - added.", &name, &date.format("%d.%m.%y").to_string());
    Ok(())
}

fn ride(conn: &mut Connection, command: Command) -> Result<()> {
    if command.category.is_none() || command.bike_id.is_none() || command.val.is_none() {
        err_exit!("Command params missed.\nExpected: `bcl add ride [category]:[bike_id] [distance]`");
    }

    let date: NaiveDate = command.date.to_naive();
    let cat: Category = get_category(conn, command.category.unwrap().as_str()).unwrap();
    let bike: Bike = get_bike(conn, cat.id, command.bike_id.unwrap()).unwrap();
    let distance: f32 = command.val.unwrap();
    let annotation: Option<String> = if command.annotation.is_empty() {
        None
    } else {
        Some(command.annotation.join(" "))
    };
    let mut tags_id: Vec<i32> = Vec::new();

    if !command.include_tags.is_empty() {
        for tag in command.include_tags {
            tags_id.push(tag_get_or_create(conn, tag.as_str()).unwrap());
        }
    }

    let tx = conn.transaction()?;

    tx.execute(
        "INSERT INTO ride (bike_id, datestamp, distance, annotation) VALUES (?1, ?2, ?3, ?4)",
        params![bike.id, date, distance, annotation]
    )?;

    let ride_id: i32 = tx.last_insert_rowid() as i32;

    for tag_id in tags_id {
        tx.execute(
            "INSERT INTO tag_to_ride (tag_id, ride_id) VALUES (?1, ?2)",
            params![tag_id, ride_id]
        )?;
    }

    tx.commit()?;

    println!("Ride {0}:{1} to {2}km from {3} - added", &cat.abbr, &command.bike_id.unwrap(), &distance, &date.format("%d.%m.%y").to_string());

    Ok(())
}
