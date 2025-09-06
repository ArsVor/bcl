use owo_colors::OwoColorize;
use rusqlite::{Connection, Result, ToSql, params, params_from_iter};

use crate::cli::structs::Command;
use crate::db::models::{BikeList, BuyList, Category, ChainLubricationList, RideList};
use crate::db::queries::get_category;
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

    let mut lub: ChainLubricationList = match (lubs.len(), id) {
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

    lub = helpers::editor::edit_lub(lub).expect("failed to edit lub");

    Ok(())
}
fn ride(conn: &mut Connection, command: Command) -> Result<()> {
    let id: Option<u32> = command.id.get();
    let mut rides: Vec<RideList> = helpers::get::ride(conn, command)?;

    let mut ride: RideList = match (rides.len(), id) {
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

    ride = helpers::editor::edit_ride(ride).expect("failed to edit lub");

    Ok(())
}
