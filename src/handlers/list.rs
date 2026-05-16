use anyhow::Result;
use owo_colors::OwoColorize;
use rusqlite::Connection;
use tabled::Table;
use tabled::settings::{Format, Modify, Style, object::Cell};

use crate::cli::structs::Command;
use crate::db::models::{BikeList, BuyList, Category, ChainLubricationList, RideList};

use super::helpers::{self, BuyResult, RideResult};
use crate::err_exit;

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
        "cat" => categories(&conn),
        "lub" => chain_lub(&conn, command),
        "ride" => ride(&conn, command),
        "tag" => tag(&conn),
        _ => {
            err_exit!(format!(
                "Have not OBJECT `{}`. Tyr `bcl list help` for more info.",
                &command.object.unwrap()
            ));
        }
    }
}

fn categories(conn: &Connection) -> Result<()> {
    let categories: Vec<Category> = helpers::get::categories(conn)?;

    if !categories.is_empty() {
        let mut table = Table::new(categories);
        table.with(Style::rounded());
        println!("{}", &table);
    } else {
        println!("{}", "Nothing found for your query.".yellow());
    }

    Ok(())
}

fn tag(conn: &Connection) -> Result<()> {
    let tags = helpers::get::tag(conn)?;

    for tag in tags {
        println!("{}", tag.as_str().green());
    }

    Ok(())
}

fn bike(conn: &Connection, command: Command) -> Result<()> {
    let bikes: Vec<BikeList> = helpers::get::bike(conn, command)?;

    if !bikes.is_empty() {
        let mut table = Table::new(bikes);
        table.with(Style::rounded());
        println!("{}", &table);
    } else {
        println!("{}", "Nothing found for your query.".yellow());
    }

    Ok(())
}

fn buy(conn: &Connection, command: Command) -> Result<()> {
    let currency: String = command.config.units.currency.clone();
    let result: BuyResult = helpers::get::buy(conn, command)?;

    let buys: Vec<BuyList> = if let helpers::BuyResult::List(buys) = result {
        buys
    } else {
        unreachable!()
    };

    if !buys.is_empty() {
        let mut table = Table::new(buys);
        table.with(
            Modify::new(Cell::new(0, 5)).with(Format::content(|_| format!("Price ({currency})"))),
        );
        table.with(Style::rounded());
        println!("{}", &table);
    } else {
        println!("{}", "Nothing found for your query.".yellow());
    }

    Ok(())
}

fn ride(conn: &Connection, command: Command) -> Result<()> {
    let distance_unit: String = command.config.units.distance.clone();
    let result: RideResult = helpers::get::ride(conn, command)?;

    let rides: Vec<RideList> = if let helpers::RideResult::List(rides) = result {
        rides
    } else {
        unreachable!()
    };

    if !rides.is_empty() {
        let mut table = Table::new(rides);
        table.with(
            Modify::new(Cell::new(0, 4))
                .with(Format::content(|_| format!("Distance ({distance_unit})"))),
        );
        table.with(Style::rounded());
        println!("{}", &table);
    } else {
        println!("{}", "Nothing found for your query.".yellow());
    }

    Ok(())
}

fn chain_lub(conn: &Connection, command: Command) -> Result<()> {
    let distance_unit: String = command.config.units.distance.clone();
    let lubs: Vec<ChainLubricationList> = helpers::get::chain_lub(conn, command)?;

    if !lubs.is_empty() {
        let mut table = Table::new(lubs);
        table.with(
            Modify::new(Cell::new(0, 4))
                .with(Format::content(|_| format!("Passed ({distance_unit})"))),
        );
        table.with(Style::rounded());
        println!("{}", &table);
    } else {
        println!("{}", "Nothing found for your query.".yellow());
    }
    Ok(())
}
