use owo_colors::OwoColorize;
use rusqlite::{Connection, Result, params};

use crate::cli::structs::Command;
use crate::db::models::{BikeInfo, BikeList, Category, CategoryInfo, ChainLubricationList};
use crate::db::queries;
use crate::err_exit;
use crate::output::{self, info};

use super::helpers;

pub fn route(conn: Connection, command: Command) -> Result<()> {
    let obj: String = command.object.unwrap();

    match obj.as_str() {
        "bike" => bike(&conn, command),
        "cat" => category(&conn, command),
        "lub" => lub(&conn, command),
        _ => Ok(()),
    }
}

fn bike(conn: &Connection, command: Command) -> Result<()> {
    let bike_id: i32 = if let Some(id) = command.real_id.get() {
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

    info::ride_info(bike);

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
        |row| row.get(0)
    )?;

    output::info::lub_info(lub, bike_name);

    Ok(())
}
