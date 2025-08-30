use super::parser;
use crate::err_exit;
use chrono::{Datelike, Local, NaiveDate};
use std::{clone::Clone, collections::HashSet};

#[derive(Debug, Clone)]
pub struct Field<T> {
    value: Option<T>,
}

#[derive(Debug, Clone)]
pub struct Date {
    pub year: Option<i32>,
    pub month: Option<u32>,
    pub day: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub funk: Field<String>,
    pub object: Field<String>,
    pub category: Field<String>,
    pub annotation: Vec<String>,
    pub date: Date,
    pub lt: Date,
    pub gt: Date,
    pub id: Field<u32>,
    pub real_id: Field<u32>,
    pub bike_id: Field<u8>,
    pub val: Field<f32>,
    pub val_lt: Field<f32>,
    pub val_gt: Field<f32>,
    pub lim: u8,
    pub include_tags: HashSet<String>,
    pub exclude_tags: HashSet<String>,
}

impl<T> Field<T> {
    fn new() -> Self {
        Field { value: None }
    }

    pub fn as_ref(&self) -> Option<&T> {
        self.value.as_ref()
    }

    pub fn unwrap_or(&self, default: T) -> T
    where
        T: Clone,
    {
        self.value.clone().unwrap_or(default)
    }

    pub fn unwrap(&self) -> T
    where
        T: Clone,
    {
        self.value.clone().unwrap()
    }

    pub fn set(&mut self, val: Option<T>) {
        self.value = val
    }

    pub fn get(&self) -> Option<T>
    where
        T: Clone,
    {
        self.value.clone()
    }

    pub fn set_or_err(&mut self, val: Option<T>, msg: &str) {
        if self.value.is_none() {
            self.value = val
        } else {
            err_exit!(msg);
        }
    }

    pub const fn is_some(&self) -> bool {
        self.value.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.value.is_none()
    }
}

impl Date {
    fn new() -> Date {
        Date {
            year: None,
            month: None,
            day: None,
        }
    }

    pub fn is_some(&self) -> bool {
        self.year.is_some() || self.month.is_some() || self.day.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.year.is_none() && self.month.is_none() && self.day.is_none()
    }

    pub fn year_or_now(&self) -> i32 {
        if self.year.is_some() {
            self.year.unwrap()
        } else {
            Local::now().year()
        }
    }

    pub fn month_or_now(&self) -> u32 {
        if self.month.is_some() {
            self.month.unwrap()
        } else {
            Local::now().month()
        }
    }

    pub fn day_or_now(&self) -> u32 {
        if self.day.is_some() {
            self.day.unwrap()
        } else {
            Local::now().day()
        }
    }

    pub fn is_valid_date(&self) -> bool {
        let year = self.year_or_now();
        let month = self.month_or_now();
        let day = self.day_or_now();

        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            let today = Local::now().naive_local().date();
            date <= today
        } else {
            false
        }
    }

    pub fn from_str(&mut self, val: &str) {
        let [year, month, day]: [&str; 3] = match val.split('-').collect::<Vec<_>>().try_into() {
            Ok(p) => p,
            Err(_) => {
                err_exit!("Invalid input format. Expected three parts separated by '-'");
            }
        };

        if !year.is_empty() {
            self.year_from_str(year);
        }
        if !month.is_empty() {
            self.month_from_str(month);
        }
        if !day.is_empty() {
            self.day_from_str(day);
        }
    }

    pub fn year_from_str(&mut self, val: &str) {
        if self.year.is_some() {
            err_exit!("Multiple year input");
        }

        let parsed_year: i32;
        let current_year = chrono::Local::now().year();

        if val == "prev" {
            parsed_year = current_year - 1;
        } else if val == "now" {
            parsed_year = current_year;
        } else if let Ok(number) = val.parse::<i32>() {
            parsed_year = number;
        } else {
            err_exit!(format!(
                "Wrong year format. Expected int, but given '{val}'."
            ));
        }

        self.year = Some(parsed_year);
    }

    pub fn month_from_str(&mut self, val: &str) {
        if self.month.is_some() {
            err_exit!("Multiple month input");
        }

        let parsed_month: u32;
        let current_month = Local::now().month();

        if val == "prev" {
            if current_month > 1 {
                parsed_month = current_month - 1;
            } else {
                parsed_month = 12;
            }
        } else if val == "now" {
            parsed_month = current_month;
        } else if let Ok(number) = val.parse::<u32>() {
            parsed_month = number;
        } else {
            err_exit!(format!(
                "Wrong month format. Expected int, but given '{val}'."
            ));
        }

        self.month = Some(parsed_month);
    }

    pub fn day_from_str(&mut self, val: &str) {
        if self.day.is_some() {
            err_exit!("Multiple day input");
        }

        let parsed_day: u32;
        let today = Local::now().naive_local().date();
        let current_day = today.day();

        if val == "prev" {
            if current_day > 1 {
                parsed_day = current_day - 1;
            } else {
                let prev_month = if today.month() > 1 {
                    today.month() - 1
                } else {
                    12
                };
                let year = if today.month() > 1 {
                    today.year()
                } else {
                    today.year() - 1
                };

                let days_in_prev_month = match prev_month {
                    1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                    4 | 6 | 9 | 11 => 30,
                    2 => {
                        if Date::is_leap_year(year) {
                            29
                        } else {
                            28
                        }
                    }
                    _ => unreachable!(),
                };

                parsed_day = days_in_prev_month;
            }
        } else if val == "now" {
            parsed_day = current_day;
        } else if let Ok(number) = val.parse::<u32>() {
            parsed_day = number;
        } else {
            err_exit!(format!(
                "Wrong day format. Expected int, but given '{val}'."
            ));
        }

        self.day = Some(parsed_day);
    }

    pub fn to_naive(&self) -> NaiveDate {
        NaiveDate::from_ymd_opt(self.year_or_now(), self.month_or_now(), self.day_or_now()).unwrap()
    }

    fn is_leap_year(year: i32) -> bool {
        NaiveDate::from_ymd_opt(year, 2, 29).is_some()
    }
}

impl Command {
    fn new() -> Command {
        Command {
            funk: Field::new(),
            object: Field::new(),
            category: Field::new(),
            annotation: Vec::new(),
            date: Date::new(),
            lt: Date::new(),
            gt: Date::new(),
            id: Field::new(),
            real_id: Field::new(),
            bike_id: Field::new(),
            val: Field::new(),
            val_lt: Field::new(),
            val_gt: Field::new(),
            lim: 10,
            include_tags: HashSet::new(),
            exclude_tags: HashSet::new(),
        }
    }

    pub fn from(mut args: Vec<String>) -> Command {
        let mut command: Command = Command::new();

        if let Ok(number) = args[0].parse::<u32>() {
            command.id.set(Some(number));
            args.remove(0);
        }

        if args.is_empty() {
            err_exit!("Nothing to do (from structs.rs)");
        }

        for arg in args.into_iter() {
            match arg.as_str() {
                "add" | "del" | "mod" | "edit" | "list" | "graph" | "sync" => {
                    command
                        .funk
                        .set_or_err(Some(arg), "multiple command input.");
                }
                "ls" => {
                    command
                        .funk
                        .set_or_err(Some("list".to_string()), "multiple command input.");
                }
                "buy" | "lub" | "ride" | "cat" | "tag" => {
                    command
                        .object
                        .set_or_err(Some(arg), "multiple object input.");
                }
                s if s.contains(':') => {
                    command = parser::named_parse(command, arg);
                }
                // after s.contains(':') is important!!!
                s if s.matches('-').count() == 2 => {
                    command.date.from_str(arg.as_str());
                }
                s if s.starts_with('_') => {
                    command
                        .funk
                        .set_or_err(Some("list".to_string()), "multiple command input");

                    let (obj, cat): (String, Option<String>) = parser::get_list_obj(arg);
                    command
                        .object
                        .set_or_err(Some(obj), "multiple object input.");

                    if cat.is_some() {
                        command
                            .category
                            .set_or_err(cat, "multiple bike type input.");
                    }
                }
                s if s.starts_with('+') => {
                    command.include_tags.insert(arg[1..].to_string());
                }
                s if s.starts_with('-') => {
                    command.exclude_tags.insert(arg[1..].to_string());
                }
                s if s.ends_with("+") => {
                    if let Ok(number) = s.trim_end_matches('+').parse::<f32>() {
                        command
                            .val_gt
                            .set_or_err(Some(number), "multiple max value input");
                    } else {
                        err_exit!(format!("Wrong format. Expected [float]+, but given {}", s));
                    }
                }
                s if s.ends_with("-") => {
                    if let Ok(number) = s.trim_end_matches('-').parse::<f32>() {
                        command
                            .val_lt
                            .set_or_err(Some(number), "multiple min value input");
                    } else {
                        err_exit!(format!("Wrong format. Expected [float]-, but given {}", s));
                    }
                }

                _ => {
                    if let Ok(number) = arg.parse::<f32>() {
                        command
                            .val
                            .set_or_err(Some(number), "Multiple value input.");
                    } else {
                        command.annotation.push(arg);
                    }
                }
            }
        }

        if !command.date.is_valid_date() {
            err_exit!("Non valid date given.");
        }

        command
    }
}
