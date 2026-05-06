use std::collections::HashSet;

use chrono::NaiveDate;
use owo_colors::OwoColorize;
use owo_colors::colors::xterm::{ElectricIndigo, VerdunGreen};
use rusqlite::{Connection, Result, ToSql, params, params_from_iter};

use crate::cli::structs::Command;
use crate::db::models::{Bike, Buy, Category, ChainLubrication, Ride};
use crate::db::queries::{delete_unused_tags, get_bike, get_category, tag_get_or_create_tx};
use crate::handlers::helpers::get::{get_bike_or_exit, get_category_or_exit};
use crate::{err_exit, suc_exit, warn};

use super::helpers;

pub fn route(mut conn: Connection, mut command: Command) -> Result<()> {
    let obj = if let Some(obj) = command.object.get() {
        obj
    } else {
        err_exit!(format!(
            "Object missed. Try `bcl {} help` for more info.",
            command.funk.unwrap()
        ));
    };

    if &obj == "tag" {
        return tag(&mut conn, command);
    }

    if command.update_data.is_none() {
        suc_exit!("'upd:' - was missed. Nothing to do!")
    }

    _ = helpers::clean_id(&conn, &mut command, obj.as_str());
    // crate::suc_exit!(format!("CLEANED ID: {:?}", command.cleaned_id));

    match obj.as_str() {
        "bike" => bike(&conn, command),
        // "buy" => buy(&mut conn, command),
        "cat" => cat(&conn, command),
        "lub" => lub(&conn, command),
        "ride" => ride(&mut conn, command),
        _ => Ok(()),
    }
}

fn bike(conn: &Connection, command: Command) -> Result<()> {
    if command.cleaned_id.len() > 1 {
        err_exit!("Only one bike can be changed at a time.");
    }

    let mut set_sql: Vec<String> = Vec::new();
    let mut dyn_params: Vec<Box<dyn ToSql>> = Vec::new();
    let bike_id: &u32 = &command.cleaned_id[0];
    let update_data: Box<Command> = command.update_data.unwrap();

    let updated_code = if let Some(abbr) = update_data.category.get() {
        let category_id: i32 = helpers::get::get_category_or_exit(conn, &abbr)?.id;
        let id_in_cat: i32 = helpers::get::get_next_id_in_cat(conn, category_id);
        let formatted: String = format!("{}:{} ", &abbr, &id_in_cat);

        set_sql.push("category_id = ?".into());
        set_sql.push("id_in_cat = ?".into());
        dyn_params.push(Box::new(category_id));
        dyn_params.push(Box::new(id_in_cat));

        formatted
    } else {
        String::new()
    };

    let updated_name = if !update_data.annotation.is_empty() {
        let name = update_data.annotation.join(" ");
        let formatted = format!("\"{}\" ", &name);

        set_sql.push("name = ?".into());
        dyn_params.push(Box::new(name));

        formatted
    } else {
        String::new()
    };

    let updated_date = if update_data.date.is_some() {
        let date: NaiveDate = update_data.date.to_naive();
        let formatted = format!("{}", &date);
        set_sql.push("datestamp = ?".into());
        dyn_params.push(Box::new(date));
        formatted
    } else {
        String::new()
    };

    if set_sql.is_empty() {
        suc_exit!("No data to change. Nothing to do!");
    }

    dyn_params.push(Box::new(*bike_id));

    conn.execute(
        &format!("UPDATE bike SET {} WHERE id = ?", &set_sql.join(", ")),
        params_from_iter(dyn_params.iter().map(|b| b.as_ref())),
    )?;

    println!(
        "{}",
        format!(
            "Bike id:\"{0}\" modified to: {1}{2}{3}",
            &bike_id, &updated_code, &updated_name, &updated_date,
        )
        .blue()
    );

    Ok(())
}

// fn buy(conn: &mut Connection, command: Command) -> Result<()> {
//     if command.val.is_none()
//         && command.date.is_none()
//         && command.category.is_none()
//         && command.bike_id.is_none()
//         && command.include_tags.is_empty()
//         && command.exclude_tags.is_empty()
//         && command.annotation.is_empty()
//     {
//         suc_exit!("Nothing to do!");
//     }
//
//     let mut cat: Option<Category> = None;
//     let mut bike: Option<Bike> = None;
//     let mut target: String = String::new();
//
//     let mut buy: Buy = conn.query_row(
//         "SELECT
//             b.id AS buy_id,
//             b.name AS name,
//             b.price As price,
//             b.datestamp AS datestamp,
//             COALESCE(GROUP_CONCAT(t.name, ', '), '') AS tags
//         FROM buy b
//         LEFT JOIN tag_to_buy tb ON tb.buy_id = b.id
//         LEFT JOIN tag t ON t.id = tb.tag_id
//         WHERE b.id = ?1
//         GROUP BY b.id",
//         params![id],
//         Buy::from_row,
//     )?;
//
//     if !command.annotation.is_empty() {
//         buy.name = command.annotation.join(" ");
//     }
//
//     if let Some(val) = command.val.get() {
//         buy.price = val
//     }
//
//     if command.date.is_some() {
//         buy.datestamp = command.date.to_naive()
//     }
//
//     if let Some(category) = command.category.get() {
//         let abbr: &str = category.as_str();
//         target.push_str(abbr);
//         target.push(':');
//         cat = Some(get_category_or_exit(conn, abbr)?);
//
//         if let Some(bike_id) = command.bike_id.get() {
//             target.push_str(&bike_id.to_string());
//             bike = Some(get_bike_or_exit(conn, abbr, bike_id)?);
//         }
//     }
//
//     let tx = conn.transaction()?;
//
//     tx.execute(
//         "UPDATE buy
//         SET
//             name = ?1,
//             price = ?2,
//             datestamp = ?3
//         WHERE id = ?4",
//         params![buy.name, buy.price, buy.datestamp, buy.id],
//     )?;
//
//     if let Some(bike_obj) = bike {
//         if let Ok(btb_id) = tx.query_row(
//             "SELECT id FROM buy_to_bike WHERE buy_id = ?1",
//             params![buy.id],
//             |row| row.get::<_, i32>(0),
//         ) {
//             tx.execute(
//                 "UPDATE buy_to_bike
//                 SET bike_id = ?1
//                 WHERE id = ?2",
//                 params![bike_obj.id, btb_id],
//             )?;
//         } else {
//             tx.execute(
//                 "INSERT INTO buy_to_bike (buy_id, bike_id) VALUES (?1, ?2)",
//                 params![buy.id, bike_obj.id],
//             )?;
//         }
//     }
//
//     if let Some(category) = cat {
//         if let Ok(btc_id) = tx.query_row(
//             "SELECT id FROM buy_to_category WHERE buy_id = ?1",
//             params![buy.id],
//             |row| row.get::<_, i32>(0),
//         ) {
//             tx.execute(
//                 "UPDATE buy_to_category
//                 SET category_id = ?1
//                 WHERE id = ?2",
//                 params![category.id, btc_id],
//             )?;
//         } else {
//             tx.execute(
//                 "INSERT INTO buy_to_category (buy_id, category_id) VALUES (?1, ?2)",
//                 params![buy.id, category.id],
//             )?;
//         }
//     }
//
//     if !command.include_tags.is_empty() {
//         let tags: Vec<&str> = buy.tags.split(", ").collect();
//         for tag_name in command.include_tags {
//             if !tags.contains(&tag_name.as_str()) {
//                 let tag_id = tag_get_or_create_tx(&tx, tag_name.as_str())?;
//                 tx.execute(
//                     "INSERT INTO tag_to_buy (tag_id, buy_id) VALUES (?1, ?2)",
//                     params![tag_id, buy.id],
//                 )?;
//             }
//         }
//     }
//
//     if !command.exclude_tags.is_empty() {
//         let mut tag_id_query: Vec<String> = vec![];
//         for tag_name in &command.exclude_tags {
//             tag_id_query.push(format!(
//                 "tag_id = (SELECT t.id FROM tag WHERE t.name = '{}')",
//                 &tag_name
//             ));
//         }
//
//         tx.execute(
//             "DELETE FROM tag_to_buy WHERE buy_id = ?1 AND ?2",
//             params![buy.id, format!("({})", tag_id_query.join(" OR "))],
//         )?;
//     }
//
//     tx.commit()?;
//
//     let deleted_tags: Vec<String> = delete_unused_tags(conn)?;
//
//     println!(
//         "{}",
//         format!(
//             "Buy id:{0} modified to {1} \"{3}\" {4} {5} tags:({2})",
//             buy.id, &target, buy.tags, buy.name, buy.price, buy.datestamp,
//         )
//         .blue()
//     );
//
//     if !deleted_tags.is_empty() {
//         println!(
//             "{}",
//             format!("Deleted tags: {}", deleted_tags.join(", "),).blue()
//         );
//     }
//
//     Ok(())
// }

fn cat(conn: &Connection, command: Command) -> Result<()> {
    if command.cleaned_id.len() > 1 {
        err_exit!("Only one category can be changed at a time.");
    }

    let mut set_sql: Vec<String> = Vec::new();
    let mut dyn_params: Vec<Box<dyn ToSql>> = Vec::new();
    let category_id = &command.cleaned_id[0];
    let update_data = command.update_data.unwrap();

    let updated_abbr = if let Some(abbr) = update_data.category.get() {
        set_sql.push("abbr = ?".into());

        let formatted = format!("{}: ", &abbr);
        dyn_params.push(Box::new(abbr));

        formatted
    } else {
        String::new()
    };

    let updated_annotation = if !update_data.annotation.is_empty() {
        let annotation = update_data.annotation.join(" ");
        let formatted = format!("\"{}\"", &annotation);

        set_sql.push("name = ?".into());
        dyn_params.push(Box::new(annotation));

        formatted
    } else {
        String::new()
    };

    if set_sql.is_empty() {
        suc_exit!("No data to change. Nothing to do!");
    }

    dyn_params.push(Box::new(*category_id));

    conn.execute(
        &format!("UPDATE category SET {} WHERE id = ?", &set_sql.join(", ")),
        params_from_iter(dyn_params.iter().map(|b| b.as_ref())),
    )?;

    println!(
        "{}",
        format!(
            "Category id:\"{0}\" modified to: {1}{2}",
            &category_id, &updated_abbr, &updated_annotation,
        )
        .blue()
    );

    Ok(())
}

fn lub(conn: &Connection, command: Command) -> Result<()> {
    let mut set_sql: Vec<String> = Vec::new();
    let mut dyn_params: Vec<Box<dyn ToSql>> = Vec::new();
    let mut bike_abbr: String = String::new();
    let mut updated_date: String = String::new();
    let mut updated_val: String = String::new();
    let mut updated_annotation: String = String::new();
    let update_data = command.update_data.unwrap();
    // println!("UPD DATA {:#?}", &update_data);

    if !update_data.annotation.is_empty() {
        let annotation: String = update_data.annotation.join(" ");
        updated_annotation = format!("\"{}\"", &annotation);
        set_sql.push("annotation = ?".into());
        dyn_params.push(Box::new(annotation));
    }

    if update_data.date.is_some() {
        let date: NaiveDate = update_data.date.to_naive();
        updated_date = format!("{} ", &date);
        set_sql.push("datestamp = ?".into());
        dyn_params.push(Box::new(date));
    }

    if update_data.bike_id.is_some() {
        let bike = get_bike_or_exit(
            conn,
            update_data.category.unwrap().as_str(),
            update_data.bike_id.unwrap(),
        )?;
        set_sql.push("bike_id = ?".into());
        dyn_params.push(Box::new(bike.id));

        bike_abbr = format!(
            "{}:{}",
            update_data.category.unwrap(),
            update_data.bike_id.unwrap()
        );
    }

    if let Some(val) = update_data.val.get() {
        updated_val = format!("{}km ", &val);
        set_sql.push("distance = ?".into());
        dyn_params.push(Box::new(val));
    }

    if set_sql.is_empty() {
        suc_exit!("Nothing to do!");
    }

    let placeholders = std::iter::repeat_n("?", command.cleaned_id.len())
        .collect::<Vec<_>>()
        .join(",");

    for id in &command.cleaned_id {
        dyn_params.push(Box::new(*id));
    }

    let sql: String = format!(
        "UPDATE chain_lubrication SET {} WHERE id IN ({})",
        set_sql.join(", "),
        placeholders
    );

    conn.execute(
        &sql,
        params_from_iter(dyn_params.iter().map(|b| b.as_ref())),
    )?;

    println!(
        "{}",
        format!(
            "Chain Lubrication id:\"{0}\" modified to: {1}{2}{3}{4}",
            &command
                .cleaned_id
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<String>>()
                .join(","),
            &bike_abbr,
            &updated_date,
            &updated_val,
            &updated_annotation
        )
        .blue()
    );

    Ok(())
}

fn ride(conn: &mut Connection, command: Command) -> Result<()> {
    let mut set_sql: Vec<String> = Vec::new();
    let mut dyn_params: Vec<Box<dyn ToSql>> = Vec::new();
    let update_data: Box<Command> = command.update_data.unwrap();

    let updated_code = match (update_data.category.get(), update_data.bike_id.get()) {
        (Some(abbr), Some(id_in_cat)) => {
            let bike_id = helpers::get::get_bike_or_exit(conn, &abbr, id_in_cat)?.id;
            let formatted: String = format!("{}:{} ", &abbr, &id_in_cat);

            set_sql.push("bike_id = ?".into());
            dyn_params.push(Box::new(bike_id));

            formatted
        }
        (None, None) => String::new(),
        _ => {
            err_exit!("Incorrect bike code format. \nExpected `[abbr]:[int]`.");
        }
    };

    let updated_annotation = if !update_data.annotation.is_empty() {
        let annotation = update_data.annotation.join(" ");
        let formatted = format!("\"{}\" ", &annotation);

        set_sql.push("annotation = ?".into());
        dyn_params.push(Box::new(annotation));

        formatted
    } else {
        String::new()
    };

    let updated_date = if update_data.date.is_some() {
        let date: NaiveDate = update_data.date.to_naive();
        let formatted = format!("{} ", &date);
        set_sql.push("datestamp = ?".into());
        dyn_params.push(Box::new(date));
        formatted
    } else {
        String::new()
    };

    let updated_val = if let Some(val) = update_data.val.get() {
        let formatted = format!("{}km ", &val);
        set_sql.push("distance = ?".into());
        dyn_params.push(Box::new(val));
        formatted
    } else {
        String::new()
    };

    if set_sql.is_empty()
        && update_data.include_tags.is_empty()
        && update_data.exclude_tags.is_empty()
    {
        suc_exit!("No data to change. Nothing to do!");
    }

    let tx = conn.transaction()?;

    if !set_sql.is_empty() {
        let placeholders = std::iter::repeat_n("?", command.cleaned_id.len())
            .collect::<Vec<_>>()
            .join(",");

        for id in &command.cleaned_id {
            dyn_params.push(Box::new(*id));
        }

        let sql: String = format!(
            "UPDATE ride SET {} WHERE id IN ({})",
            set_sql.join(", "),
            placeholders
        );

        tx.execute(
            &sql,
            params_from_iter(dyn_params.iter().map(|b| b.as_ref())),
        )?;
    }

    if !update_data.include_tags.is_empty() {
        let mut tag_id_set: Vec<i32> = Vec::new();

        for tag_name in &update_data.include_tags {
            tag_id_set.push(tag_get_or_create_tx(&tx, tag_name)?);
        }

        let mut stmt = tx.prepare(
            "INSERT OR IGNORE INTO tag_to_ride (tag_id, ride_id)
             VALUES (?, ?)",
        )?;

        for ride_id in &command.cleaned_id {
            for tag_id in &tag_id_set {
                stmt.execute(params![tag_id, ride_id])?;
            }
        }
    }

    if !update_data.exclude_tags.is_empty() {
        let mut sct_stmt = tx.prepare("SELECT id FROM tag WHERE name = ?")?;

        let tag_ids: HashSet<i32> = update_data
            .exclude_tags
            .iter()
            .filter_map(|name| sct_stmt.query_row([name], |row| row.get(0)).ok())
            .collect();

        let mut del_stmt =
            tx.prepare("DELETE FROM tag_to_ride WHERE tag_id = ? AND ride_id = ?")?;

        for ride_id in &command.cleaned_id {
            for tag in &tag_ids {
                del_stmt.execute(params![tag, ride_id])?;
            }
        }
    }

    tx.commit()?;

    let deleted_tags: Vec<String> = delete_unused_tags(conn)?;

    let included_tags = update_data
        .include_tags
        .into_iter()
        .map(|t| format!("+{t} "))
        .collect::<Vec<_>>()
        .join(",");

    let excluded_tags = update_data
        .exclude_tags
        .into_iter()
        .map(|t| format!("-{t} "))
        .collect::<Vec<_>>()
        .join(", ");

    let cleaned_id_str: String = command
        .cleaned_id
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<String>>()
        .join(",");

    println!(
        "{}",
        format!(
            "Ride id:({0}) modified to: {1}{2}{3}{4}{5}{6}",
            &cleaned_id_str,
            &updated_code,
            &updated_date,
            &updated_val,
            &included_tags,
            &excluded_tags,
            &updated_annotation,
        )
        .blue()
    );

    if !deleted_tags.is_empty() {
        println!(
            "{}",
            format!("Deleted tags: {}", deleted_tags.join(", "),).blue()
        );
    }

    Ok(())
}

fn tag(conn: &mut Connection, command: Command) -> Result<()> {
    if command.include_tags.is_empty() || command.exclude_tags.is_empty() {
        err_exit!("Command params missed.\nExpected: `bcl mod tag [-old_tag] [+new_tag]`");
    } else if command.exclude_tags.len() != 1 {
        err_exit!("Only one tag can be changed at a time.");
    } else if command.include_tags.len() > 1 {
        err_exit!("Multiple [+new_tag] given.\nExpected: `bcl mod tag [-old_tag] [+new_tag]`");
    }

    let old_tag: &String = command.exclude_tags.iter().last().unwrap();
    let new_tag: &String = command.include_tags.iter().last().unwrap();

    let rows_affected = conn.execute(
        "UPDATE tag
             SET name = ?1
             WHERE name = ?2",
        params![new_tag, old_tag],
    )?;

    if rows_affected == 0 {
        warn!(format!("tag +{} is not found.", &old_tag));
    } else {
        println!(
            "Tag modified: {}",
            format!("+{}  +{}", &old_tag, &new_tag).blue()
        )
    }

    Ok(())
}
