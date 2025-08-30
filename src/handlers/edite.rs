use rusqlite::{Connection, Result};

use crate::cli::structs::Command;
use crate::err_exit;

pub fn rote(mut conn: Connection, command: Command) -> Result<()> {
    let obj = command.object.unwrap();
    match obj.as_str() {
        "bike" => bike(&conn, command),
        _ => Ok(()),
    }
}

fn bike(conn: &Connection, command: Command) -> Result<()> {
    if command.id.is_none() {
        err_exit!(
            "Command params missed.\nExpected: `bcl [dynamic id]/id:[static id]` mod [PARAMS]"
        );
    }

    Ok(())
}
