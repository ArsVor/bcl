use owo_colors::OwoColorize;
use rusqlite::{Connection, Result, ToSql, params, params_from_iter};

use crate::cli::structs::Command;
use crate::db::models::{BikeList, BuyList, ChainLubricationList, RideList};
use crate::db::queries::tag_del_if_unused;
use crate::{err_exit, suc_exit};

use super::helpers::{self, BuyResult, RideResult};

pub fn route(mut conn: Connection, mut command: Command) -> Result<()> {
    let obj = if let Some(obj) = command.object.get() {
        obj
    } else {
        err_exit!(format!(
            "Object missed. Try `bcl {} help` for more info.",
            command.funk.unwrap()
        ));
    };

    if command.raw_self_id.is_empty() && command.raw_hash_id.is_empty() && command.bike_id.is_none() {
        match obj.as_str() {
            "tag" => {},
            "cat" => {
                err_exit!("Command params missed.\nExpected: `bcl del cat id:[ID]`");
            },
            _ => {
                err_exit!(format!("Command params missed.\nExpected: `bcl del {} id:[ID]/[#] {}`.", &obj, "{OPT}"));
            }
        }
    };

    _ = helpers::clean_id(&conn, &mut command, obj.as_str());
    // suc_exit!(format!("CLEANED ID: {:?}", command.cleaned_id));

    match obj.as_str() {
        "bike" => bike(&conn, command),
        "buy" => buy(&mut conn, command),
        "cat" => category(&conn, command),
        "lub" => chain_lub(&conn, command),
        "ride" => ride(&mut conn, command),
        "tag" => tag(&conn, command),
        _ => Ok(()),
    }
}

fn bike(conn: &Connection, command: Command) -> Result<()> {
    let mut sql: String = 
        "DELETE
        FROM bike
        WHERE 
        ".to_string();
    let mut where_sql: Vec<String> = vec![];
    let mut dyn_params: Vec<Box<dyn ToSql>> = Vec::new();

    for id in command.cleaned_id.clone() {
        where_sql.push(format!("id = ?{}", where_sql.len() + 1));
        dyn_params.push(Box::new(id));
    }

    sql.push_str(where_sql.join(" OR ").as_str());

    let result = conn.execute(
        &sql, 
        params_from_iter(dyn_params.iter().map(|b| b.as_ref()))
    );

    match result {
        Ok(_) => {
            println!(
                "{}",
                format!(
                    "Bike id:{} deleted successfully.", 
                    command.cleaned_id
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(","))
                .blue()
            );
            Ok(())
        }
        Err(rusqlite::Error::SqliteFailure(err, _))
            if err.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            err_exit!("You can not delete a bike that has rides or chain lubrication.");
        }
        Err(e) => {
            err_exit!(&e);
        }
    }
}

fn buy(conn: &mut Connection, command: Command) -> Result<()> {
    let mut sql = 
        "DELETE 
        FROM buy
        WHERE
        ".to_string();
    let mut where_sql: Vec<String> = vec![];
    let mut dyn_params: Vec<Box<dyn ToSql>> = Vec::new();

    for id in command.cleaned_id.clone() {
        where_sql.push(format!("id = ?{}", where_sql.len() + 1));
        dyn_params.push(Box::new(id));
    }

    sql.push_str(where_sql.join(" OR ").as_str());

    _ = conn.execute(
        &sql, 
        params_from_iter(dyn_params.iter().map(|b| b.as_ref()))
    );

    let deleted_tags: Vec<String> = tag_del_if_unused(conn)?;

    println!(
        "{}", 
        format!(
            "buy id:{} deleted successfully.", 
            command.cleaned_id
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(",")).blue()
        );

    if !deleted_tags.is_empty() {
        println!(
            "{}",
            format!("Deleted tags: {}", deleted_tags.join(", "),).blue()
        );
    }

    Ok(())
}

fn category(conn: &Connection, command: Command) -> Result<()> {
    let id: i32 = if let Some(id) = command.absolute_id.get() {
        id as i32
    } else {
        err_exit!("Command params missed.\nExpected: `bcl del cat id:[ID]`");
    };

    let result = conn.execute("DELETE FROM category WHERE id = ?1", params![id]);

    match result {
        Ok(_) => {
            println!(
                "{}",
                format!("Category id:{} deleted successfully.", &id).blue()
            );
            Ok(())
        }
        Err(rusqlite::Error::SqliteFailure(err, _))
            if err.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            err_exit!("You cannot delete a non-empty category.");
        }
        Err(e) => {
            err_exit!(&e);
        }
    }
}

fn chain_lub(conn: &Connection, command: Command) -> Result<()> {
    if command.id.is_none() && command.absolute_id.is_none() {
        err_exit!("Command params missed.\nExpected: `bcl del lub id:[ID]/[#] {OPT}`");
    }

    let id: i32 = if let Some(absolute_id) = command.absolute_id.get() {
        absolute_id as i32
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

    conn.execute("DELETE FROM chain_lubrication WHERE id = ?1", params![id])?;

    println!(
        "{}",
        format!("Chain lubrication id:{} deleted successfully.", &id).blue()
    );

    Ok(())
}

fn ride(conn: &mut Connection, command: Command) -> Result<()> {
    let mut sql = 
        "DELETE 
        FROM buy
        WHERE
        ".to_string();
    let mut where_sql: Vec<String> = vec![];
    let mut dyn_params: Vec<Box<dyn ToSql>> = Vec::new();

    for id in command.cleaned_id.clone() {
        where_sql.push(format!("id = ?{}", where_sql.len() + 1));
        dyn_params.push(Box::new(id));
    }

    sql.push_str(where_sql.join(" OR ").as_str());

    _ = conn.execute(
        &sql, 
        params_from_iter(dyn_params.iter().map(|b| b.as_ref()))
    );

    let deleted_tags: Vec<String> = tag_del_if_unused(conn)?;

    println!(
        "{}",
        format!(
            "Ride id:{} deleted successfully.", 
            command.cleaned_id
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(",")).blue()
    );

    if !deleted_tags.is_empty() {
        println!(
            "{}",
            format!("Deleted tags: {}", deleted_tags.join(", "),).blue()
        );
    }

    Ok(())
}

fn tag(conn: &Connection, command: Command) -> Result<()> {
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
    println!("TAGS TO DELEE: {:?}", &tags_to_delete);

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
