pub mod cli;
pub mod db;
pub mod handlers;
mod makros;

use std::env::args;

use cli::structs::Command;
use db::helpers::open_connection_with_fk;
use rusqlite::Connection;

fn main() {
    let mut args: Vec<String> = args().collect();
    args.remove(0);
    // println!("{:?}", &args);
    if !args.is_empty() {
        let conn: Connection = open_connection_with_fk("./bcl.db").unwrap();
        let command: Command = Command::from(args);
        // println!("{:?}", &command);
        let funk = command.funk.unwrap();
        let _ = match funk.as_str() {
            "add" => handlers::add::route(conn, command),
            "list" => handlers::list::route(conn, command),
            "mod" => {
                if command.real_id.is_some() || command.object.unwrap() == "tag" {
                    handlers::update::rote(conn, command)
                } else {
                    handlers::edite::rote(conn, command)
                }
            }
            _ => Ok(()),
        };
        // println!("{:?}", db::queries::get_category(&conn, "G"))
        // println!("Is year? - {:?}", &command.date.year.is_some());
        // println!("Year is - {:?}", &command.date.year_or_now());
        // println!("Now year is - {:?}", &command.date.year);
        // println!("Is date a valid? - {:?}", &command.date.is_valid_date());
        // println!("{:?}", &command.funk.is_some());
        // println!("{:?}", &command.val.unwrap_or(0.0));
        // println!("done")
    } else {
        err_exit!("Nothing to do (from main.rs)");
        // потім реалізую логіку виводу help
    }
}
