use std::slice::Iter;

use owo_colors::OwoColorize;
use rusqlite::{Connection, Result, ToSql, params, params_from_iter};

use crate::cli::structs::Command;
use crate::db::models::RideList;
use crate::db::queries::tag_del_if_unused;
use crate::{err_exit, suc_exit};

use super::helpers;

pub fn route(mut conn: Connection, command: Command) -> Result<()> {
    let obj = command.object.unwrap();
    match obj.as_str() {
        "bike" => bike(&mut conn, command),
        "buy" => buy(&mut conn, command),
        "cat" => category(&mut conn, command),
        "lub" => chain_lub(&mut conn, command),
        "ride" => ride(&mut conn, command),
        "tag" => tag(&mut conn, command),
        _ => Ok(()),
    }
}

fn bike(conn: &mut Connection, command: Command) -> Result<()> {
    Ok(())
}
fn buy(conn: &mut Connection, command: Command) -> Result<()> {
    Ok(())
}
fn category(conn: &mut Connection, command: Command) -> Result<()> {
    Ok(())
}
fn chain_lub(conn: &mut Connection, command: Command) -> Result<()> {
    Ok(())
}
fn ride(conn: &mut Connection, command: Command) -> Result<()> {
    if command.id.is_none() && command.real_id.is_none() {
        err_exit!("Command params missed.\nExpected: `bcl del ride id:[stat_id]/[dyn_id] {OPT}`.");
    }

    let id: i32;
    let mut tags: Vec<String> = Vec::new();
    let mut deleted_tags: Vec<String> = Vec::new();

    if let Some(real_id) = command.real_id.get() {
        id = real_id as i32;
    } else {
        let dyn_id: usize = command.id.unwrap() as usize;
        let rides: Vec<RideList> = helpers::get::ride(conn, command)?;

        let ride: RideList = rides.get(dyn_id - 1).cloned().unwrap_or_else(|| {
            err_exit!("Ride for your request was not found.");
        });
        id = ride.ride_id;
        tags.extend(ride.tags.split(", ").map(|s| s.to_string()));
    }

    conn.execute("DELETE FROM ride WHERE id = ?1", params![id])?;

    for tag_name in tags {
        if let Some(tag_name) = tag_del_if_unused(conn, tag_name.as_str())? {
            deleted_tags.push(tag_name);
        }
    }

    println!(
        "{}",
        format!("Ride id:{} deleted successfully.", &id).blue()
    );

    if !deleted_tags.is_empty() {
        println!(
            "{}",
            format!("Deleted tags: {}", deleted_tags.join(", "),).blue()
        );
    }

    Ok(())
}

fn tag(conn: &mut Connection, command: Command) -> Result<()> {
    let mut tags_to_delete: Vec<String> = Vec::new();

    if !command.include_tags.is_empty() {
        tags_to_delete.extend(command.include_tags.clone());
    }

    if !command.exclude_tags.is_empty() {
        tags_to_delete.extend(command.exclude_tags.clone());
    }

    if !command.annotation.is_empty() {
        tags_to_delete.append(&mut command.annotation.clone());
    }

    if tags_to_delete.is_empty() {
        suc_exit!("Nothing to do!");
    }

    let del_all: bool;

    println!(
        "{}: Deleting a tag will also remove it from all associated objects.",
        "WARNING".to_string().yellow(),
    );
    println!("This action cannot be undone.\n");
    if tags_to_delete.len() > 1 {
        println!("Do you want to continue? [y/N/a]");
        println!("y - yes (apply to all)");
        println!("n - no (default)");
        println!("a - ask before every deletion");
    } else {
        println!("Do you want to continue? [y/N]");
        println!("y - yes");
        println!("n - no (default)");
    }
    let mut choice: String = String::new();
    std::io::stdin().read_line(&mut choice).unwrap();

    match choice.trim().to_lowercase().as_str() {
        "y" => del_all = true,
        "a" => del_all = false,
        _ => {
            println!("{}: Deletion canceled.", "INFO".blue());
            std::process::exit(0)
        }
    }

    if tags_to_delete.len() == 1 {
        conn.execute(
            "DELETE FROM tag WHERE name = ?1",
            params![tags_to_delete.join("")],
        )?;
    }

    let mut delete_sql: String = "DELETE FROM tag WHERE name IN (".to_string();
    let mut dyn_params: Vec<Box<dyn ToSql>> = Vec::new();
    let mut num: u8 = 1;
    for tag_name in tags_to_delete {
        if !del_all {
            println!("\nDelete tag \"{}\"? [y/N]", &tag_name);
            println!("y - yes");
            println!("n - no (default)");

            let mut choice: String = String::new();
            std::io::stdin().read_line(&mut choice).unwrap();

            if choice.trim().to_lowercase().as_str() != "y" {
                continue;
            }
        }

        delete_sql.push_str(format!("?{}, ", &num).as_str());
        dyn_params.push(Box::new(tag_name));
        num += 1;
    }

    _ = delete_sql.pop();
    _ = delete_sql.pop();
    delete_sql.push(')');

    if !dyn_params.is_empty() {
        conn.execute(
            &delete_sql,
            params_from_iter(dyn_params.iter().map(|b| b.as_ref())),
        )?;
        println!("{}: Tags deleted successfully.", "INFO".blue());
    } else {
        println!("{}: No tags for deletion.", "INFO".blue())
    }

    Ok(())
}
