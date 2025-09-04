use owo_colors::OwoColorize;
use rusqlite::{Connection, Result, params};

use crate::cli::structs::Command;
use crate::db::models::{BikeList, BuyList, Category, ChainLubricationList, RideList};
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
    let mut bikes: Vec<BikeList> = helpers::get::bike(conn, command)?;

    let mut bike: BikeList = match (bikes.len(), id) {
        (0, _) => {
            err_exit!("Bike for your request was not found.");
        },
        (1, None) => bikes.pop().unwrap(),
        (_, Some(dyn_id)) => bikes.get(dyn_id as usize - 1).cloned().unwrap_or_else(||{
            err_exit!("Bike for your request was not found.");
        }),
        _ => {
            err_exit!("Not enough params. Can't select 1 bike.");
        }
    };

    bike = helpers::editor::edit_bike(bike).expect("failed to edit buy");
    // println!("Now name: {}", &bike.bike);

    Ok(())
}

fn buy(conn: &mut Connection, command: Command) -> Result<()> {
    let id: Option<u32> = command.id.get();
    let mut buys: Vec<BuyList> = helpers::get::buy(conn, command)?;

    let mut buy: BuyList = match (buys.len(), id) {
        (0, _) => {
            err_exit!("Buy for your request was not found.");
        },
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
        },
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
