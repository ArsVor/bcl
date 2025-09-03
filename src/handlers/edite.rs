use rusqlite::{params, Connection, Result};

use crate::cli::structs::Command;
use crate::db::models::{BikeList, BuyList, ChainLubricationList, RideList};
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
    let mut bike_list: Vec<BikeList> = helpers::get::bike(conn, command)?;

    if bike_list.is_empty() {
        err_exit!("Bike for your request was not found.");
    }
        
    let mut bike: BikeList = if bike_list.len() > 1 {
        let dyn_id = if let Some(id) = id {
            id as usize
        } else {
            err_exit!("Not enoughs params. Can't select 1 bike.");
        };
        bike_list.get(dyn_id - 1).cloned().unwrap_or_else(|| {
            err_exit!("Bike for your request was not found.");
        })
    } else {
        bike_list.pop().unwrap()
    };

    bike = helpers::editor::edit_bike(bike).expect("failed to edit buy");

    Ok(())
}

fn buy(conn: &mut Connection, command: Command) -> Result<()> {
    if command.id.is_none() && command.real_id.is_none() {
        err_exit!(
            "Command params missed.\nExpected: `bcl [dynamic id]/id:[static id] edit buy [PARAMS]`."
        );
    }
    let real_id: Option<u32> = command.real_id.get();

    let mut buy_list: Vec<BuyList> = if command.real_id.is_some() {
        helpers::get::buy(conn, command)?
    } else {
        let dyn_id: usize = command.id.unwrap() as usize;
        let buys: Vec<BuyList> = helpers::get::buy(conn, command)?;

        let buy: BuyList = buys.get(dyn_id - 1).cloned().unwrap_or_else(|| {
            err_exit!("buy for your request was not found.");
        });
        vec![buy]
    };

    let mut buy: BuyList = if let Some(buy) = buy_list.pop() {
        buy
    } else {
        err_exit!(format!("buy id:{} does not exist.", real_id.unwrap()));
    };

    buy = helpers::editor::edit_buy(buy).expect("failed to edit buy");

    Ok(())
}
fn category(conn: &Connection, command: Command) -> Result<()> {
    if command.id.is_none() && command.real_id.is_none() {
        err_exit!(
            "Command params missed.\nExpected: `bcl [dynamic id]/id:[static id] edit cat [PARAMS]`."
        );
    }

    let id:i32 = if let Some(real_id) = command.real_id.get() {
        real_id as i32
    } else {
        command.id.unwrap() as i32
    };


    Ok(())
}
fn chain_lub(conn: &Connection, command: Command) -> Result<()> {
    if command.id.is_none() && command.real_id.is_none() {
        err_exit!(
            "Command params missed.\nExpected: `bcl [dynamic id]/id:[static id] edit lub [PARAMS]`."
        );
    }

    let id: i32 = if let Some(real_id) = command.real_id.get() {
        real_id as i32
    } else {
        let dyn_id: usize = command.id.unwrap() as usize;
        let lubs: Vec<ChainLubricationList> = helpers::get::chain_lub(conn, command)?;

        let id: i32 = lubs
            .get(dyn_id - 1)
            .cloned()
            .unwrap_or_else(|| {
                err_exit!("Chain lubrication for your request was not found.");
            })
            .lub_id;
        id
    };

    Ok(())
}
fn ride(conn: &mut Connection, command: Command) -> Result<()> {
    if command.id.is_none() && command.real_id.is_none() {
        err_exit!(
            "Command params missed.\nExpected: `bcl [dynamic id]/id:[static id] edit ride [PARAMS]`."
        );
    }

    let id: i32 = if let Some(real_id) = command.real_id.get() {
        real_id as i32
    } else {
        let dyn_id: usize = command.id.unwrap() as usize;
        let rides: Vec<RideList> = helpers::get::ride(conn, command)?;

        let id: i32 = rides.get(dyn_id - 1).cloned().unwrap_or_else(|| {
            err_exit!("Ride for your request was not found.");
        }).ride_id;
        id
    };

    Ok(())
}
