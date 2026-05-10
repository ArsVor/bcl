use std::path::PathBuf;

use anyhow::Result;
use lazy_regex::regex_is_match;
use rusqlite::Connection;

use super::structs::Command;
use crate::db::helpers::open_connection_with_fk;
use crate::err_exit;
use crate::init::{get_config, init_paths};

pub fn get_bicycle_types() -> Result<Vec<String>> {
    // REG DEV mod
    // let db: PathBuf = PathBuf::from("./bcl.db");
    // let conn: Connection = open_connection_with_fk(&db)?;
    // REGEND DEV mod

    // REG RELEASE mod
    let paths = init_paths()?;

    let config_file: PathBuf = paths.config_dir.join("config.toml");
    let config = get_config(&config_file)?;

    let conn: Connection = open_connection_with_fk(&config.database.path)?;
    // REGEND RELEASE mod

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
    } else if key.contains("graph") {
        command.output.set(Some(key.to_string()));

        if !val.is_empty() {
            command.group_by.set(Some(val.to_string()))
        };
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
                if command.id.is_some()
                    || !command.raw_hash_id.is_empty()
                    || !command.raw_self_id.is_empty()
                {
                    err_exit!("Input # or ID, not both.");
                }

                command.raw_self_id = multiple_id_pars(val.to_string());
            }
            _ => {
                err_exit!(format!("Unexpected key: '{key}'."));
            }
        }
    } else {
        match key {
            "lim" => {
                command.lim = 0;
            }
            _ => {
                err_exit!(format!("Unexpected key: '{key}'."));
            }
        }
    }

    command
}

pub fn multiple_id_pars(val: String) -> Vec<u32> {
    let coma_parts: Vec<&str> = val.split(",").collect();
    let mut id_vec: Vec<u32> = vec![];

    fn add_range(range: String, id_vec: &mut Vec<u32>) {
        let range_splited: Vec<&str> = range.split("..").collect();
        let start: u32 = range_splited[0].parse().unwrap();
        let stop: u32 = range_splited[1].parse().unwrap();

        if start > stop {
            err_exit!(format!(
                "incorrect id range format. Expected: `min..max`, but {}..{} given",
                &start, &stop
            ));
        }

        for i in start..=stop {
            id_vec.push(i);
        }
    }

    for part in coma_parts {
        if !part.is_empty() {
            match part {
                s if regex_is_match!(r"^\d+\.\.\d+$", s) => add_range(s.to_string(), &mut id_vec),
                s if regex_is_match!(r"^\d+$", s) => id_vec.push(s.parse().unwrap()),
                _ => {
                    err_exit!("incorrect multilpe id syntax");
                }
            }
        }
    }

    id_vec
}

pub fn update_data_parse(data_str: String) -> Option<Box<Command>> {
    let mut args: Vec<String> = data_str
        .split(" ")
        .filter(|p| !p.is_empty())
        .map(|p| p.to_string())
        .collect();

    if args.is_empty() {
        return None;
    } else {
        args.push("#~~#".to_string());

        if regex_is_match!(r"^\d+$", args[0].as_str()) {
            args[0] = format!("val:{}", args[0]);
        }
    }

    Some(Box::new(Command::from(args)))
}
