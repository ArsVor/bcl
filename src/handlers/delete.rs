use owo_colors::OwoColorize;
use rusqlite::{params, params_from_iter, Connection, Result, ToSql};

use crate::cli::structs::Command;
use crate::{err_exit, suc_exit};

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

fn bike(conn: &mut Connection, command: Command) -> Result<()> {Ok(())}
fn buy(conn: &mut Connection, command: Command) -> Result<()> {Ok(())}
fn category(conn: &mut Connection, command: Command) -> Result<()> {Ok(())}
fn chain_lub(conn: &mut Connection, command: Command) -> Result<()> {Ok(())}
fn ride(conn: &mut Connection, command: Command) -> Result<()> {Ok(())}
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

    println!("{}: Deleting a tag will also remove it from all associated objects.",
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
            params![tags_to_delete.join("")])?;
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
        conn.execute(&delete_sql, params_from_iter(dyn_params.iter().map(|b| b.as_ref())))?;
        println!("{}: Tags deleted successfully.", "INFO".blue());
    } else {
        println!("{}: No tags for deletion.", "INFO".blue())
    }

    Ok(())
}
