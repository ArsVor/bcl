use owo_colors::OwoColorize;
use rusqlite::{Connection, Result, ToSql, params, params_from_iter};

use crate::cli::structs::Command;
use crate::db::queries::delete_unused_tags;
use crate::{err_exit, suc_exit};

use super::helpers;

pub fn route(mut conn: Connection, mut command: Command) -> Result<()> {
    let obj = if let Some(obj) = command.object.get() {
        obj
    } else {
        err_exit!(format!(
            "Object missed. Try `bcl {} help` for more info.",
            command.funk.unwrap()
        ));
    };

    if command.raw_self_id.is_empty() {
        match obj.as_str() {
            "bike" => {
                if command.bike_id.is_none() {
                    err_exit!(format!(
                        "Command params missed.\nExpected: `bcl del {} id:[ID]/[#]/[Code] {}`.",
                        &obj, "{OPT}"
                    ));
                }
            }
            "cat" => {
                err_exit!("Command params missed.\nExpected: `bcl del cat id:[ID]`");
            }
            "tag" => {}
            _ => {
                if command.raw_hash_id.is_empty() {
                    err_exit!(format!(
                        "Command params missed.\nExpected: `bcl del {} id:[ID]/[#] {}`.",
                        &obj, "{OPT}"
                    ));
                }
            }
        }
    };

    _ = helpers::clean_id(&conn, &mut command, obj.as_str());
    // suc_exit!(format!("CLEANED ID: {:?}", command.cleaned_id));

    match obj.as_str() {
        "bike" => bike(&mut conn, command),
        "buy" => buy(&mut conn, command),
        "cat" => category(&mut conn, command),
        "lub" => chain_lub(&mut conn, command),
        "ride" => ride(&mut conn, command),
        "tag" => tag(&conn, command),
        _ => Ok(()),
    }
}

fn delete_with_id_set(
    conn: &mut Connection,
    id_set: Vec<u32>,
    table: String,
) -> Result<usize, rusqlite::Error> {
    let mut sql: String = format!(
        "DELETE
        FROM {}
        WHERE 
        ",
        &table
    );
    let mut where_sql: Vec<String> = vec![];
    let mut dyn_params: Vec<Box<dyn ToSql>> = Vec::new();

    for id in id_set {
        where_sql.push(format!("id = ?{}", where_sql.len() + 1));
        dyn_params.push(Box::new(id));
    }

    sql.push_str(where_sql.join(" OR ").as_str());

    conn.execute(
        &sql,
        params_from_iter(dyn_params.iter().map(|b| b.as_ref())),
    )
}

fn bike(conn: &mut Connection, command: Command) -> Result<()> {
    let result = delete_with_id_set(conn, command.cleaned_id.clone(), String::from("bike"));

    match result {
        Ok(_) => {
            println!(
                "{}",
                format!(
                    "Bike id:{} deleted successfully.",
                    command
                        .cleaned_id
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>()
                        .join(",")
                )
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
    _ = delete_with_id_set(conn, command.cleaned_id.clone(), String::from("buy"));

    let deleted_tags: Vec<String> = delete_unused_tags(conn)?;

    println!(
        "{}",
        format!(
            "buy id:{} deleted successfully.",
            command
                .cleaned_id
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(",")
        )
        .blue()
    );

    if !deleted_tags.is_empty() {
        println!(
            "{}",
            format!("Deleted tags: {}", deleted_tags.join(", "),).blue()
        );
    }

    Ok(())
}

fn category(conn: &mut Connection, command: Command) -> Result<()> {
    let result = delete_with_id_set(conn, command.cleaned_id.clone(), String::from("category"));

    match result {
        Ok(_) => {
            println!(
                "{}",
                format!(
                    "Category id:{} deleted successfully.",
                    command
                        .cleaned_id
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>()
                        .join(",")
                )
                .blue()
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

fn chain_lub(conn: &mut Connection, command: Command) -> Result<()> {
    _ = delete_with_id_set(
        conn,
        command.cleaned_id.clone(),
        String::from("chain_lubrication"),
    );

    println!(
        "{}",
        format!(
            "Chain lubrication id:{} deleted successfully.",
            command
                .cleaned_id
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(",")
        )
        .blue()
    );

    Ok(())
}

fn ride(conn: &mut Connection, command: Command) -> Result<()> {
    _ = delete_with_id_set(conn, command.cleaned_id.clone(), String::from("ride"));

    let deleted_tags: Vec<String> = delete_unused_tags(conn)?;

    println!(
        "{}",
        format!(
            "Ride id:{} deleted successfully.",
            command
                .cleaned_id
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(",")
        )
        .blue()
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
