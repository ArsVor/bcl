use super::structs::Command;
use crate::err_exit;

pub fn get_bicycle_types() -> Vec<String> {
    let bicycle_types: Vec<String> = vec!["G".to_string(), "R".to_string(), "M".to_string()];
    bicycle_types
}

pub fn get_list_obj(arg: String) -> (String, Option<String>) {
    let bicycle_types: Vec<String> = get_bicycle_types();
    let val: String = arg[1..].to_string();

    if bicycle_types.contains(&val) {
        ("bike".to_string(), Some(val))
    } else {
        (val, None)
    }
}

pub fn is_bike_type(val: &str) -> bool {
    let bicycle_types: Vec<String> = get_bicycle_types();
    bicycle_types.contains(&val.to_string())
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
