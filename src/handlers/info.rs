use owo_colors::OwoColorize;
use rusqlite::{Connection, Result, params};

use crate::cli::structs::Command;
use crate::db::models::{BikeInfo, BikeList};
use crate::db::queries;
use crate::err_exit;

use super::helpers;

pub fn route(conn: Connection, command: Command) -> Result<()> {
    let obj: String = command.object.unwrap();

    match obj.as_str() {
        "bike" => bike(&conn, command),
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

    let after_lub_distance: f32 = bike.after_lub_distance;
    let msg: String = format!(
        "Without chain lubrication, passed: {}km",
        &after_lub_distance
    );

    println!("{}", format!("\n~~ {} ~~", &bike.name).green());
    println!(
        "{}",
        format!("Category:         {}", &bike.category).green()
    );
    println!("{}", format!("ID:               {}", &bike.id).green());
    println!("{}", format!("Bike code:        {}", &bike.code).green());
    println!(
        "{}",
        format!("Added:            {}", &bike.add_date).green()
    );
    println!(
        "{}",
        format!("Total spend:      {}", &bike.total_spend).green()
    );
    println!(
        "{}",
        format!("Ride count:       {}", &bike.ride_count).green()
    );
    if let Some(date) = bike.last_ride {
        println!(
            "{}",
            format!("Total distance:   {}km", &bike.total_distance).green()
        );
        println!("{}", format!("Last ride:        {}", &date).green());
        println!(
            "{}",
            format!("    distance:     {}km", &bike.last_distance).green()
        );
    }
    if let Some(date) = bike.maintenance {
        println!("{}", format!("Last maintenance: {}", &date).green());
    }
    if let Some(date) = bike.chain_lub {
        println!("{}", format!("Last chain lub:   {}", &date).green());
    }
    if after_lub_distance > 0.00 {
        if after_lub_distance > 200.00 {
            println!("{}", msg.red());
        } else if after_lub_distance > 150.00 {
            println!("{}", msg.yellow());
        } else {
            println!("{}", msg.green());
        }
    };
    Ok(())
}
