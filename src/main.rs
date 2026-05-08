pub mod cli;
pub mod db;
pub mod handlers;
mod makros;
pub mod output;

use std::env::args;

use cli::structs::Command;
use db::helpers::open_connection_with_fk;
use rusqlite::Connection;

fn main() {
    let mut args: Vec<String> = args().collect();
    args.remove(0);
    if !args.is_empty() {
        let conn: Connection = open_connection_with_fk("./bcl.db").unwrap();
        let command: Command = Command::from(args);

        let funk = command.funk.unwrap();
        let result = match funk.as_str() {
            "add" => handlers::add::route(conn, command),
            "del" => handlers::delete::route(conn, command),
            "edit" => handlers::edit::route(conn, command),
            "info" => handlers::info::route(conn, command),
            "list" => handlers::list::route(conn, command),
            "mod" => handlers::update::route(conn, command),
            _ => Ok(()),
        };

        if let Err(e) = result {
            err_exit!(&e);
        }
    } else {
        err_exit!("Nothing to do (from main.rs)");
        // потім реалізую логіку виводу help
    }
}
