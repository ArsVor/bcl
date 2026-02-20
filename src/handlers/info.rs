use owo_colors::OwoColorize;
use rusqlite::{Connection, Result, params};

use crate::cli::structs::Command;
use crate::db::models::{
    BikeInfo, BikeList, BuyInfo, Category, CategoryInfo, ChainLubricationList, RideInfo,
};
use crate::db::queries;
use crate::handlers::helpers::{BuyResult, RideResult};
use crate::handlers::structs::{BuysInfoReport, RidesInfoReport};
use crate::output;
use crate::{err_exit, suc_exit};

use super::helpers;

pub fn route(conn: Connection, mut command: Command) -> Result<()> {
    let obj = if let Some(obj) = command.object.get() {
        obj
    } else {
        err_exit!(format!(
            "Object missed. Try `bcl {} help` for more info.",
            command.funk.unwrap()
        ));
    };

    if !command.raw_self_id.is_empty() {
        command.cleaned_id = command.raw_self_id.clone();
    } else if !command.raw_hash_id.is_empty() {
        command.cleaned_id = command.raw_hash_id.clone();
    }

    match obj.as_str() {
        "bike" => bike(&conn, command),
        "buy" => buy(&conn, command),
        "cat" => category(&conn, command),
        "lub" => lub(&conn, command),
        "ride" => ride(&conn, command),
        _ => Ok(()),
    }
}

fn bike(conn: &Connection, command: Command) -> Result<()> {
    let bike_id: i32 = if let Some(id) = command.absolute_id.get() {
        id as i32
    } else if let Some(id) = command.bike_id.get() {
        if let Some(abbr) = command.category.get() {
            queries::get_bike(conn, &abbr, id)?.id
        } else {
            err_exit!("Bike for your request was not found.");
        }
    } else {
        let id: Option<u32> = command.id.get();
        let mut bikes: Vec<BikeList> = helpers::get::bike(conn, command)?;
        let bike: BikeList = match (bikes.len(), id) {
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
        bike.bike_id
    };

    let bike: BikeInfo = helpers::get::bike_info(conn, bike_id)?;

    output::info::bike_info(bike);

    Ok(())
}

fn buy(conn: &Connection, command: Command) -> Result<()> {
    let result: BuyResult = helpers::get::buy(conn, command.clone())?;

    let buys: Vec<BuyInfo> = if let helpers::BuyResult::Info(buys) = result {
        buys
    } else {
        unreachable!()
    };

    match buys.len() {
        0 => {
            suc_exit!("Buys for your request was not found.");
        }
        1 => output::info::buy_info_single(&buys[0]),
        _ => {
            let report: BuysInfoReport = BuysInfoReport::from(buys, &command);

            if command.output.is_none() {
                output::info::buy_info(report);
            } else {
                output::graph::buy_graph(report);
            }
        }
    }

    Ok(())
}

fn category(conn: &Connection, command: Command) -> Result<()> {
    let category: Category = helpers::get::category_with_params(conn, command)?;
    let cat_info: CategoryInfo = helpers::get::category_info(conn, category.id)?;
    output::info::category_info(cat_info);

    Ok(())
}

fn lub(conn: &Connection, command: Command) -> Result<()> {
    let id: Option<u32> = command.id.get();
    let mut lubs: Vec<ChainLubricationList> = helpers::get::chain_lub(conn, command)?;
    let lub: ChainLubricationList = match (lubs.len(), id) {
        (0, _) => {
            err_exit!("Chain lubrication for your request was not found.");
        }
        (1, None) => lubs.pop().unwrap(),
        (_, Some(dyn_id)) => lubs.get(dyn_id as usize - 1).cloned().unwrap_or_else(|| {
            err_exit!("Chain lubrication for your request was not found.");
        }),
        _ => {
            err_exit!("Not enough params. Can't select 1 bike.");
        }
    };

    let code: Vec<String> = lub.bike.clone().split(":").map(|s| s.to_string()).collect();
    let bike_id: i32 = code[1].parse().unwrap();

    let bike_name: String = conn.query_row(
        "SELECT
            b.name
        FROM category c
        LEFT JOIN bike b ON b.category_id = c.id
        WHERE c.abbr = ?1 AND b.id_in_cat = ?2",
        params![code[0], bike_id],
        |row| row.get(0),
    )?;

    output::info::lub_info(lub, bike_name);

    Ok(())
}

fn ride(conn: &Connection, command: Command) -> Result<()> {
    let result: RideResult = helpers::get::ride(conn, command.clone())?;

    let rides: Vec<RideInfo> = if let helpers::RideResult::Info(rides) = result {
        rides
    } else {
        unreachable!()
    };

    // println!("Rides: {:?}", &rides);
    match rides.len() {
        0 => {
            suc_exit!("Rides for your request was not found.");
        }
        1 => output::info::ride_info_single(&rides[0]),
        _ => {
            let report: RidesInfoReport = RidesInfoReport::from(rides, &command);
            // println!("{:?}", &report);
            if command.output.is_none() {
                output::info::ride_info(report);
            } else {
                output::graph::ride_graph(report);
            };
        }
    }

    Ok(())
}
