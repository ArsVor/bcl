use owo_colors::OwoColorize;
use rusqlite::{Connection, Result};
use tabled::Table;
use tabled::settings::Style;

use crate::cli::structs::Command;
use crate::db::models::{BikeList, BuyList, Category, ChainLubricationList, RideList};

use super::helpers;

pub fn route(conn: Connection, command: Command) -> Result<()> {
    let obj = command.object.unwrap();
    match obj.as_str() {
        "bike" => bike(&conn, command),
        "buy" => buy(&conn, command),
        "cat" => categories(&conn),
        "lub" => chain_lub(&conn, command),
        "ride" => ride(&conn, command),
        "tag" => tag(&conn),
        _ => Ok(()),
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
    let buys: Vec<BuyList> = helpers::get::buy(conn, command)?;

    if !buys.is_empty() {
        let mut table = Table::new(buys);
        table.with(Style::rounded());
        println!("{}", &table);
    } else {
        println!("{}", "Nothing found for your query.".yellow());
    }

    Ok(())
}

fn ride(conn: &Connection, command: Command) -> Result<()> {
    let rides: Vec<RideList> = helpers::get::ride(conn, command)?;

    if !rides.is_empty() {
        let mut table = Table::new(rides);
        table.with(Style::rounded());
        println!("{}", &table);
    } else {
        println!("{}", "Nothing found for your query.".yellow());
    }

    Ok(())
}

fn chain_lub(conn: &Connection, command: Command) -> Result<()> {
    let lubs: Vec<ChainLubricationList> = helpers::get::chain_lub(conn, command)?;

    if !lubs.is_empty() {
        let mut table = Table::new(lubs);
        table.with(Style::rounded());
        println!("{}", &table);
    } else {
        println!("{}", "Nothing found for your query.".yellow());
    }
    Ok(())
}
