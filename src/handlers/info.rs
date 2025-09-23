use owo_colors::OwoColorize;
use rusqlite::{Connection, Result, params};

use crate::cli::structs::Command;
use crate::db::models::{BikeInfo, BikeList, Category, CategoryInfo};
use crate::db::queries;
use crate::err_exit;
use crate::output::{self, info};

use super::helpers;

pub fn route(conn: Connection, command: Command) -> Result<()> {
    let obj: String = command.object.unwrap();

    match obj.as_str() {
        "bike" => bike(&conn, command),
        "cat" => category(&conn, command),
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
