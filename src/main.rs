pub mod cli;
pub mod db;
pub mod handlers;
pub mod init;
mod makros;
pub mod output;

use std::{env::args, path::PathBuf};

use anyhow::Result;
use cli::structs::Command;
use db::helpers::open_connection_with_fk;
use init::{create_default_config_file, get_config, init_paths};
use rusqlite::Connection;

fn main() -> Result<()> {
    let mut args: Vec<String> = args().collect();
    args.remove(0);
    if !args.is_empty() {
        // REG DEV mode
        let paths = init_paths("dev")?;
        // REGEND DEV mode

        // REG RELEASE mod
        // let paths = init_paths("release")?;
        // REGEND RELEASE mod

        let config_file: PathBuf = paths.config_dir.join("config.toml");
        let data_dir: PathBuf = paths.data_dir;

        if !config_file.exists() {
            create_default_config_file(&config_file, &data_dir)?;
        }

        let config = get_config(&config_file)?;

        let conn: Connection = open_connection_with_fk(&config.database.path)?;
        let command: Command = Command::from(&conn, args)?;
        // println!("Command: {:#?}", &command);
        // suc_exit!("DONE!!!");

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

    Ok(())
}
// #endregion
