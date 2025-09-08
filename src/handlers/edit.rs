use std::collections::HashSet;

use owo_colors::OwoColorize;
use rusqlite::{Connection, Result, ToSql, params, params_from_iter};

use crate::cli::structs::Command;
use crate::db::models::{BikeList, BuyList, Category, ChainLubricationList, RideList};
use crate::db::queries::{get_bike, get_category, tag_del_if_unused, tag_get_or_create};
use crate::err_exit;

use super::helpers;

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

fn bike(conn: &Connection, command: Command) -> Result<()> {
    let id = command.id.get();
    let cat: i32;
    let mut id_in_cat: Option<i32> = None;
    let mut bikes: Vec<BikeList> = helpers::get::bike(conn, command)?;

    let mut bike: BikeList = match (bikes.len(), id) {
        (0, _) => {
            err_exit!("Bike for your request was not found.");
        }
        (1, None) => bikes.pop().unwrap(),
        (_, Some(dyn_id)) => bikes.get(dyn_id as usize - 1).cloned().unwrap_or_else(|| {
            err_exit!("Bike for your request was not found.");
        }),
        _ => {
            err_exit!("Not enough params. Can't select 1 bike.");
        }
    };

    let bike_cod: String = bike.code.clone();

    bike = helpers::editor::edit_bike(bike).expect("failed to edit buy");

    let mut dyn_params: Vec<Box<dyn ToSql>> = vec![Box::new(&bike.name), Box::new(bike.added)];
    let mut sql: String = "
        UPDATE bike
        SET
            name = ?1,
            datestamp = ?2
    "
    .to_string();

    if bike_cod != bike.code {
        let cod_parts: Vec<&str> = bike.code.splitn(2, ":").collect();
        if let Ok(num) = cod_parts[1].parse::<i32>() {
            id_in_cat = Some(num)
        };
        cat = get_category(conn, cod_parts[0])?.id;

        if let Some(id_in_cat) = id_in_cat {
            let exist: bool = conn.query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM bike WHERE category_id = ?1 AND id_in_cat = ?2
                )",
                params![cat, id_in_cat],
                |row| row.get(0),
            )?;
            if !exist {
                sql.push_str(&format!(", category_id = ?{}", dyn_params.len() + 1));
                dyn_params.push(Box::new(cat));
                sql.push_str(&format!(", id_in_cat = ?{}", dyn_params.len() + 1));
                dyn_params.push(Box::new(id_in_cat));
            } else {
                err_exit!(format!("Bike {} is already exist.", &bike.code));
            };
        }
    }

    sql.push_str(&format!(" WHERE id = ?{}", dyn_params.len() + 1));
    dyn_params.push(Box::new(bike.bike_id));

    conn.execute(
        &sql,
        params_from_iter(dyn_params.iter().map(|b| b.as_ref())),
    )?;

    println!(
        "{}",
        format!(
            "Bike - id:{} set to: '{} {} {}'",
            &bike.bike_id, &bike.code, &bike.name, &bike.added
        )
        .blue()
    );

    Ok(())
}

fn buy(conn: &mut Connection, command: Command) -> Result<()> {
    let id: Option<u32> = command.id.get();
    let mut buys: Vec<BuyList> = helpers::get::buy(conn, command)?;

    let mut buy: BuyList = match (buys.len(), id) {
        (0, _) => {
            err_exit!("Buy for your request was not found.");
        }
        (1, None) => buys.pop().unwrap(),
        (_, Some(dyn_id)) => buys.get(dyn_id as usize - 1).cloned().unwrap_or_else(|| {
            err_exit!("Buy for your request was not found.");
        }),
        _ => {
            err_exit!("Not enough params. Can't select 1 buy.");
        }
    };

    buy = helpers::editor::edit_buy(buy).expect("failed to edit buy");
    // println!("Now date: {}; name: {}", &buy.date, &buy.name);

    Ok(())
}

fn category(conn: &Connection, command: Command) -> Result<()> {
    let mut category: Category = helpers::get::category_with_params(conn, command)?;

    category = helpers::editor::edit_cat(category).expect("failed to edit cat");

    Ok(())
}

fn chain_lub(conn: &Connection, command: Command) -> Result<()> {
    let id: Option<u32> = command.id.get();
    let mut lubs: Vec<ChainLubricationList> = helpers::get::chain_lub(conn, command)?;

    let lub_def: ChainLubricationList = match (lubs.len(), id) {
        (0, _) => {
            err_exit!("Chain lubrication for your request was not found.");
        }
        (1, None) => lubs.pop().unwrap(),
        (_, Some(dyn_id)) => lubs.get(dyn_id as usize - 1).cloned().unwrap_or_else(|| {
            err_exit!("Chain lubrication for your request was not found.");
        }),
        _ => {
            err_exit!("Not enough params. Can't select 1 lub.");
        }
    };

    let lub: ChainLubricationList = helpers::editor::edit_lub(lub_def.clone()).expect("failed to edit lub");

    let annotation: String = if !lub.annotation.is_empty() {
        format!("\"{}\"", &lub.annotation)
    } else {
        String::new()
    };

    let mut sql: String = "
        UPDATE chain_lubrication
        SET
            datestamp = ?1,
            annotation = ?2
    "
    .to_string();
    let mut dyn_params: Vec<Box<dyn ToSql>> = vec![Box::new(lub.date), Box::new(lub.annotation)];

    if lub.bike != lub_def.bike {
        let bike_code: Vec<String> = lub
            .bike
            .clone()
            .split(":")
            .map(|s| s.to_string())
            .collect();
        let abbr: &str = bike_code[0].as_str();
        let id_in_cat: u8 = bike_code[1].parse().unwrap_or_else(|_| {
            err_exit!(format!(
                "Incorrect bike code format. \nExpected `[abbr]:[int]`, but given - {}",
                &lub.bike
            ));
        });
        let bike_id: i32 = get_bike(conn, abbr, id_in_cat)?.id;

        sql.push_str(format!(", bike_id = ?{}", dyn_params.len() + 1).as_str());
        dyn_params.push(Box::new(bike_id));
    }

    sql.push_str(format!(" WHERE id = ?{}", dyn_params.len() + 1).as_str());
    dyn_params.push(Box::new(lub.lub_id));

    conn.execute(&sql, params_from_iter(dyn_params.iter().map(|b| b.as_ref())))?;

    println!(
        "{}", 
        format!(
            "Chain Lubrication - id:\"{0}\" modified to {1} {2} {3}",
            lub.lub_id, lub.bike, lub.date, &annotation,
        )
        .blue(),
    );

    Ok(())
}

fn ride(conn: &mut Connection, command: Command) -> Result<()> {
    let id: Option<u32> = command.id.get();
    let mut rides: Vec<RideList> = helpers::get::ride(conn, command)?;
    let mut deleted_tags: Vec<String> = Vec::new();

    let ride_def: RideList = match (rides.len(), id) {
        (0, _) => {
            err_exit!("Ride for your request was not found.");
        }
        (1, None) => rides.pop().unwrap(),
        (_, Some(dyn_id)) => rides.get(dyn_id as usize - 1).cloned().unwrap_or_else(|| {
            err_exit!("Ride for your request was not found.");
        }),
        _ => {
            err_exit!("Not enough params. Can't select 1 ride.");
        }
    };

    let ride: RideList = helpers::editor::edit_ride(ride_def.clone()).expect("failed to edit ride");

    let annotation: String = if !ride.annotation.is_empty() {
        format!("\"{}\"", &ride.annotation)
    } else {
        String::new()
    };

    let tags_str: String = ride
        .tags
        .split(", ")
        .map(|s| {
            let mut t = String::from(s);
            t.insert(0, '+');
            t
        })
        .collect::<Vec<String>>()
        .join(", ");

    let mut sql: String = "
        UPDATE ride
        SET
            datestamp = ?1,
            distance = ?2,
            annotation = ?3
    "
    .to_string();
    let mut dyn_params: Vec<Box<dyn ToSql>> = vec![
        Box::new(ride.date),
        Box::new(ride.distance),
        Box::new(&ride.annotation),
    ];

    if ride.bike != ride_def.bike {
        let bike_code: Vec<String> = ride
            .bike
            .clone()
            .split(":")
            .map(|s| s.to_string())
            .collect();
        let abbr: &str = bike_code[0].as_str();
        let id_in_cat: u8 = bike_code[1].parse().unwrap_or_else(|_| {
            err_exit!(format!(
                "Incorrect bike code format. \nExpected `[abbr]:[int]`, but given - {}",
                &ride.bike
            ));
        });
        let bike_id: i32 = get_bike(conn, abbr, id_in_cat)?.id;

        sql.push_str(format!(", bike_id = ?{}", dyn_params.len() + 1).as_str());
        dyn_params.push(Box::new(bike_id));
    }

    sql.push_str(format!(" WHERE id = ?{}", dyn_params.len() + 1).as_str());
    dyn_params.push(Box::new(ride.ride_id));

    conn.execute(
        &sql,
        params_from_iter(dyn_params.iter().map(|b| b.as_ref())),
    )?;

    if ride.tags != ride_def.tags {
        let tags_to_add: HashSet<String> = helpers::tags_diff(&ride.tags, &ride_def.tags);
        let tags_to_del: HashSet<String> = helpers::tags_diff(&ride_def.tags, &ride.tags);

        if !tags_to_add.is_empty() {
            for tag_name in tags_to_add {
                let tag_id = tag_get_or_create(conn, tag_name.as_str())?;
                conn.execute(
                    "INSERT INTO tag_to_ride (tag_id, ride_id) VALUES (?1, ?2)",
                    params![tag_id, ride.ride_id],
                )?;
            }
        }

        if !tags_to_del.is_empty() {
            for tag_name in tags_to_del {
                if let Ok(tag_id) = conn.query_row(
                    "SELECT id FROM tag WHERE name = ?1",
                    params![tag_name],
                    |row| row.get::<_, i32>(0),
                ) {
                    conn.execute(
                        "DELETE FROM tag_to_ride WHERE tag_id = ?1 AND ride_id = ?2",
                        params![tag_id, ride.ride_id],
                    )?;
                }
                if let Some(tag_name) = tag_del_if_unused(conn, tag_name.as_str())? {
                    deleted_tags.push(tag_name);
                }
            }
        }
    }

    println!(
        "{}",
        format!(
            "Ride - id:\"{0}\" modified to {1} {2} {3} {4} {5}",
            ride.ride_id, ride.bike, ride.date, ride.distance, &tags_str, &annotation,
        )
        .blue()
    );
    if !deleted_tags.is_empty() {
        println!(
            "{}",
            format!("Deleted tags: {}", deleted_tags.join(", "),).blue()
        );
    }

    Ok(())
}
