use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;
use owo_colors::OwoColorize;
use rusqlite::{Connection, MappedRows, Result, Row, ToSql, params_from_iter};
use tabled::Table;
use tabled::settings::{Style, style};

use crate::cli::structs::{Command, Field};
use crate::db::models::{Bike, BikeList, Buy, BuyList, Category, RideList};
use crate::db::queries::{get_bike, get_category, get_included_excluded};
use crate::err_exit;

use super::helpers::get_date;

pub fn route(mut conn: Connection, command: Command) -> Result<()> {
    let obj = command.object.unwrap();
    match obj.as_str() {
        "bike" => bike(&conn, command),
        "buy" => buy(&conn, command),
        "cat" => categories(&conn),
        "ride" => ride(&conn, command),
        "tag" => tag(&conn),
        _ => Ok(()),
    }
}

fn categories(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("SELECT * FROM category")?;
    let category_iter = stmt.query_map([], |row| Category::from_row(row))?;

    let mut categories: Vec<Category> = Vec::new();

    for category in category_iter {
        categories.push(category?);
    }

    let mut table = Table::new(categories);
    table.with(Style::rounded());
    println!("{}", &table);

    Ok(())
}

fn tag(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("SELECT name FROM tag")?;
    let tag_iter = stmt.query_map([], |row| row.get::<_, String>(0))?;

    for tag in tag_iter {
        let tag = tag?;
        println!("{}", tag.as_str().green());
    }

    Ok(())
}

fn bike(conn: &Connection, command: Command) -> Result<()> {
    let bikes: Vec<BikeList> = if let Some(cat_str) = command.category.get() {
        let cat = get_category(conn, &cat_str)?;
        let mut stmt = conn.prepare(
            "
            SELECT 
                ROW_NUMBER() OVER (ORDER BY b.id) AS row_num,
                b.id, 
                c.abbr, 
                b.name, 
                b.datestamp  
            FROM bike b 
                JOIN category c ON c.id = b.category_id
            WHERE category_id = ?1
        ",
        )?;
        stmt.query_map([cat.id], |row| BikeList::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?
    } else {
        let mut stmt = conn.prepare(
            "
            SELECT 
                ROW_NUMBER() OVER (ORDER BY b.id) AS row_num,
                b.id, 
                c.abbr, 
                b.name, 
                b.datestamp  
            FROM bike b 
                JOIN category c ON c.id = b.category_id
        ",
        )?;
        stmt.query_map([], |row| BikeList::from_row(row))?
            .collect::<Result<Vec<_>, _>>()?
    };

    let mut table = Table::new(bikes);
    table.with(Style::rounded());
    println!("{}", &table);

    Ok(())
}

fn buy(conn: &Connection, command: Command) -> Result<()> {
    let mut select_sql: String = "
        SELECT 
            b.id AS buy_id,
            b.name,
            b.price,
            b.datestamp,
            COALESCE(GROUP_CONCAT(t.name, ', '), '') AS tags,
            COALESCE(bk.name, c.name, '') AS bike_or_category
        FROM buy b
        LEFT JOIN tag_to_buy tb ON tb.buy_id = b.id
        LEFT JOIN tag t ON t.id = tb.tag_id
        LEFT JOIN buy_to_category bc ON bc.buy_id = b.id
        LEFT JOIN category c ON c.id = bc.category_id
        LEFT JOIN buy_to_bike bbk ON bbk.buy_id = b.id
        LEFT JOIN bike bk ON bk.id = bbk.bike_id
    "
    .to_string();
    let mut where_sql: Vec<String> = vec![];
    let mut dyn_params: Vec<Box<dyn ToSql>> = Vec::new();
    let eq: bool = command.date.day.is_some();

    let date: Option<NaiveDate> = get_date(command.date.clone());
    let date_lt: Option<NaiveDate> = get_date(command.lt.clone());
    let date_gt: Option<NaiveDate> = get_date(command.gt.clone());

    let val: Field<f32> = command.val.clone();
    let val_gt: Field<f32> = command.val_gt.clone();
    let val_lt: Field<f32> = command.val_lt.clone();

    let bike_id: Option<u8> = command.bike_id.get();
    let category: Option<String> = command.category.get();

    if val.is_some() {
        where_sql.push(format!("b.price = ?{}", where_sql.len() + 1));
        dyn_params.push(Box::new(val.unwrap()));
    } else {
        if val_gt.is_some() {
            where_sql.push(format!("b.price > ?{}", where_sql.len() + 1));
            dyn_params.push(Box::new(val_gt.unwrap()));
        }
        if val_lt.is_some() {
            where_sql.push(format!("b.price < ?{}", where_sql.len() + 1));
            dyn_params.push(Box::new(val_lt.unwrap()));
        }
    }

    if let Some(date) = date {
        match eq {
            true => where_sql.push(format!("b.datestamp = ?{}", where_sql.len() + 1)),
            false => where_sql.push(format!("b.datestamp >= ?{}", where_sql.len() + 1)),
        }
        println!("date is some");
        dyn_params.push(Box::new(date));
    } else {
        if let Some(date_gt) = date_gt {
            where_sql.push(format!("b.datestamp > ?{}", where_sql.len() + 1));
            dyn_params.push(Box::new(date_gt));
        }
        if let Some(date_lt) = date_lt {
            where_sql.push(format!("b.datestamp < ?{}", where_sql.len() + 1));
            dyn_params.push(Box::new(date_lt));
        }
    }

    if let Some(bike_id) = bike_id {
        let bike: Bike = get_bike(conn, category.unwrap().as_str(), bike_id)?;
        where_sql.push(format!("bk.id = ?{}", where_sql.len() + 1));
        dyn_params.push(Box::new(bike.id));
    } else if let Some(category) = category {
        where_sql.push(format!("c.abbr = ?{}", where_sql.len() + 1));
        dyn_params.push(Box::new(category));
    }

    if !command.annotation.is_empty() {
        let name: String = command.annotation.join(" ");
        where_sql.push(format!("b.name LIKE ?{}", where_sql.len() + 1));
        dyn_params.push(Box::new(format!("%{}%", &name)));
    }

    if !where_sql.is_empty() {
        select_sql.push_str(" WHERE ");
        select_sql.push_str(&where_sql.join(" AND "));
    }

    select_sql.push_str("GROUP BY b.id, b.name, b.price, b.datestamp ORDER BY b.id");

    let mut stmt = conn.prepare(&select_sql)?;
    let buys_iter = stmt.query_map(
        params_from_iter(dyn_params.iter().map(|b| b.as_ref())),
        |row| BuyList::from_row(row),
    )?;

    let (include_id, exclude_id): (HashSet<i32>, HashSet<i32>) =
        get_included_excluded(conn, command, "buy")?;

    let mut buys: Vec<BuyList> = Vec::new();
    let mut num = 1;
    for buy_result in buys_iter {
        let mut buy: BuyList = buy_result?;
        if (include_id.contains(&buy.buy_id) || include_id.is_empty())
            && !exclude_id.contains(&buy.buy_id)
        {
            buy.id = num;
            num += 1;
            buys.push(buy);
        }
    }

    if !buys.is_empty() {
        let mut table = Table::new(buys);
        table.with(Style::rounded());
        println!("{}", &table);
    } else {
        println!("{}", "Nothing found for your query.".yellow());
    }

    Ok(())
}

fn ride(conn: &Connection, command: Command) -> Result<()> {
    let mut select_sql: String = "
        SELECT
            r.id,
            c.abbr,
            b.name,
            r.datestamp,
            r.distance,
            COALESCE(r.annotation, '') AS ann,
            COALESCE(GROUP_CONCAT(t.name, ', '), '') AS tags
        FROM ride r
        LEFT JOIN tag_to_ride tr ON r.id = tr.ride_id
        LEFT JOIN tag t ON tr.tag_id = t.id
        LEFT JOIN bike b ON r.bike_id = b.id
        LEFT JOIN category c ON b.category_id = c.id
    "
    .to_string();

    let mut where_sql: Vec<String> = vec![];
    let mut dyn_params: Vec<Box<dyn ToSql>> = Vec::new();
    let eq: bool = command.date.day.is_some();

    let date: Option<NaiveDate> = get_date(command.date.clone());
    let date_lt: Option<NaiveDate> = get_date(command.lt.clone());
    let date_gt: Option<NaiveDate> = get_date(command.gt.clone());

    let val: Field<f32> = command.val.clone();
    let val_gt: Field<f32> = command.val_gt.clone();
    let val_lt: Field<f32> = command.val_lt.clone();

    let bike_id: Option<u8> = command.bike_id.get();
    let category: Option<String> = command.category.get();

    if val.is_some() {
        where_sql.push(format!("r.distance = ?{}", where_sql.len() + 1));
        dyn_params.push(Box::new(val.unwrap()));
    } else {
        if val_gt.is_some() {
            where_sql.push(format!("r.distance > ?{}", where_sql.len() + 1));
            dyn_params.push(Box::new(val_gt.unwrap()));
        }
        if val_lt.is_some() {
            where_sql.push(format!("r.distance < ?{}", where_sql.len() + 1));
            dyn_params.push(Box::new(val_lt.unwrap()));
        }
    }

    if let Some(date) = date {
        match eq {
            true => where_sql.push(format!("r.datestamp = ?{}", where_sql.len() + 1)),
            false => where_sql.push(format!("r.datestamp >= ?{}", where_sql.len() + 1)),
        }
        dyn_params.push(Box::new(date));
    } else {
        if let Some(date_gt) = date_gt {
            where_sql.push(format!("r.datestamp > ?{}", where_sql.len() + 1));
            dyn_params.push(Box::new(date_gt));
        }
        if let Some(date_lt) = date_lt {
            where_sql.push(format!("r.datestamp < ?{}", where_sql.len() + 1));
            dyn_params.push(Box::new(date_lt));
        }
    }

    if let Some(bike_id) = bike_id {
        let bike: Bike = get_bike(conn, category.unwrap().as_str(), bike_id)?;
        where_sql.push(format!("b.id = ?{}", where_sql.len() + 1));
        dyn_params.push(Box::new(bike.id));
    } else if let Some(category) = category {
        where_sql.push(format!("c.abbr = ?{}", where_sql.len() + 1));
        dyn_params.push(Box::new(category));
    }

    if !command.annotation.is_empty() {
        let name: String = command.annotation.join(" ");
        where_sql.push(format!("r.annotation LIKE ?{}", where_sql.len() + 1));
        dyn_params.push(Box::new(format!("%{}%", &name)));
    }

    if !where_sql.is_empty() {
        select_sql.push_str(" WHERE ");
        select_sql.push_str(&where_sql.join(" AND "));
    }
    select_sql.push_str("GROUP BY r.id ORDER BY r.id");

    let (include_id, exclude_id): (HashSet<i32>, HashSet<i32>) =
    get_included_excluded(conn, command, "ride")?;

    let mut stmt = conn.prepare(&select_sql)?;
    let rides_iter = stmt.query_map(params_from_iter(dyn_params.iter().map(|b| b.as_ref())), |row| RideList::from_row(row))?;

    let mut rides: Vec<RideList> = Vec::new();
    let mut num: i32 = 1;
    for ride_result in rides_iter {
        let mut ride = ride_result?;
        if (include_id.contains(&ride.ride_id) || include_id.is_empty()) && !exclude_id.contains(&ride.ride_id) {
            ride.id = num;
            num += 1;
            rides.push(ride);
        }
    }

    if !rides.is_empty() {
        let mut table = Table::new(rides);
        table.with(Style::rounded());
        println!("{}", &table);
    } else {
        println!("{}", "Nothing found for your query.".yellow());
    }

    Ok(())
}
