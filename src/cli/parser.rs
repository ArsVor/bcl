use rusqlite::{Connection, Result};

use super::structs::Command;
use crate::db::helpers::open_connection_with_fk;
use crate::err_exit;

pub fn get_bicycle_types() -> Result<Vec<String>> {
    let conn: Connection = open_connection_with_fk("./bcl.db").unwrap();
    
    let mut stmt = conn.prepare("SELECT abbr FROM category")?;
    let bicycle_types: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(|res| res.unwrap())
            .collect();

    Ok(bicycle_types)
}

pub fn get_list_obj(arg: String) -> (String, Option<String>) {
    let val: String = arg[1..].to_string();

    match get_bicycle_types() {
        Ok(bicycle_types) if bicycle_types.contains(&val) => ("bike".to_string(), Some(val)),
        Ok(_) => (val, None),
        Err(e) => {
            err_exit!(&e);
        }
    }
}

pub fn is_bike_type(val: &str) -> bool {
    match get_bicycle_types() {
        Ok(bicycle_types) => bicycle_types.contains(&val.to_string()),
        Err(e) => {
            err_exit!(&e);
        }
    }
}

pub fn named_parse(mut command: Command, arg: String) -> Command {
    let parsed_arg: Vec<&str> = arg.split(":").collect();

    if parsed_arg.len() != 2 {
        err_exit!(format!("Bad syntax - '{arg}'!"));
    }

    let (key, val): (&str, &str) = (parsed_arg[0], parsed_arg[1]);

    if is_bike_type(key) {
        if !val.is_empty() {
            if let Ok(number) = val.parse::<u8>() {
                command
                    .bike_id
                    .set_or_err(Some(number), "multiple bike id input.");
            } else {
                err_exit!(format!(
                    "Wrong value of '{key}'. Expected integer, but given '{val}'"
                ));
            }
        }
        command
            .category
            .set_or_err(Some(key.to_string()), "multiple bike type input.");
    } else if !val.is_empty() {
        match key {
            "year" => {
                command.date.year_from_str(val);
            }
            "month" => {
                command.date.month_from_str(val);
            }
            "day" => {
                command.date.day_from_str(val);
            }
            "date" => {
                command.date.from_str(val);
            }
            "lt" => {
                command.lt.from_str(val);
            }
            "gt" => {
                command.gt.from_str(val);
            }
            "cat" | "bike" => {
                command
                    .object
                    .set_or_err(Some(key.to_string()), "multiple object input.");
                command
                    .category
                    .set_or_err(Some(val.to_string()), "multiple bike type input.");
            }
            "val" => {
                if let Ok(number) = val.parse::<f32>() {
                    command
                        .val
                        .set_or_err(Some(number), "multiple value input.");
                } else {
                    err_exit!(format!(
                        "Wrong value of '{key}'. Expected float, but given '{val}'"
                    ));
                }
            }
            "lim" => {
                if let Ok(number) = val.parse::<u8>() {
                    command.lim = number;
                } else {
                    err_exit!(format!(
                        "Wrong value of '{key}'. Expected int from 0 to 255, but given '{val}'"
                    ));
                }
            }
            "id" => {
                if command.id.is_some() {
                    err_exit!("Input dynamic id or static id, not both.");
                }
                if let Ok(number) = val.parse::<u32>() {
                    command
                        .real_id
                        .set_or_err(Some(number), "multiple static id input");
                } else {
                    err_exit!(format!(
                        "Wrong value of '{key}'. Expected int, but given '{val}'"
                    ));
                }
            }
            _ => {
                err_exit!(format!("Unexpected key - '{key}'."));
            }
        }
    } else {
        match key {
            "lim" => {
                command.lim = 0;
            }
            _ => {
                err_exit!(format!("Unexpected key - '{key}'."));
            }
        }
    }

    command
}
