use std::collections::HashSet;

use lazy_regex::regex_is_match;
use owo_colors::OwoColorize;
use rusqlite::{Connection, Error, ErrorCode, Result, ToSql, params, params_from_iter};

use crate::cli::structs::Command;
use crate::db::models::{BikeList, BuyList, Category, ChainLubricationList, RideList};
use crate::db::queries::{
    delete_unused_tags, get_bike, get_category, tag_get_or_create_tx,
};
use crate::{err_exit, warn};

use super::helpers::{self, BuyResult, RideResult};

pub fn route(mut conn: Connection, mut command: Command) -> Result<()> {
    let obj = if let Some(obj) = command.object.get() {
        obj
    } else {
        err_exit!(format!(
            "Object missed. Try `bcl {} help` for more info.",
            command.funk.unwrap()
        ));
    };

    _ = helpers::clean_id(&conn, &mut command, obj.as_str());

    match obj.as_str() {
        "bike" => bike(&conn, command),
        "buy" => buy(&mut conn, command),
        "cat" => category(&conn, command),
        "lub" => chain_lub(&conn, command),
        "ride" => ride(&mut conn, command),
        _ => Ok(()),
    }
}

fn bike(conn: &Connection, command: Command) -> Result<()> {
    let bikes: Vec<BikeList> = helpers::get::bike(conn, command)?;

    for mut bike in bikes {
        let bike_cod: String = bike.code.clone();

        bike = helpers::editor::edit_bike(bike).expect("failed to edit bike");

        let mut dyn_params: Vec<Box<dyn ToSql>> = vec![Box::new(&bike.name), Box::new(bike.added)];
        let mut sql: String = "
            UPDATE bike
            SET
                name = ?1,
                datestamp = ?2
        "
        .to_string();

        if bike_cod != bike.code {
            let cod_parts: Vec<&str> = bike.code.splitn(2, ":").collect();
            let id_in_cat: Option<i32> = cod_parts[1].parse::<i32>().ok();
            let Some(category) = get_category(conn, cod_parts[0])? else {
                warn!(format!("category - '{}' does not exist.", cod_parts[0]));
                continue;
            };
            let cat = category.id;

            let Some(id_in_cat) = id_in_cat else {
                warn!(
                    format!(
                        "Incorrect bike code format. Skepped. \nExpected `[abbr]:[int]`, but given - {}.", 
                        &bike.code
                        )
                    );
                continue;
            };

            let exist: bool = conn.query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM bike WHERE category_id = ?1 AND id_in_cat = ?2
                )",
                params![cat, id_in_cat],
                |row| row.get(0),
            )?;

            if !exist {
                sql.push_str(&format!(", category_id = ?{}", dyn_params.len() + 1));
                dyn_params.push(Box::new(cat));
                sql.push_str(&format!(", id_in_cat = ?{}", dyn_params.len() + 1));
                dyn_params.push(Box::new(id_in_cat));
            } else {
                warn!(format!("Bike {} is already exist.", &bike.code));
                continue;
            };
        }

        sql.push_str(&format!(" WHERE id = ?{}", dyn_params.len() + 1));
        dyn_params.push(Box::new(bike.bike_id));

        conn.execute(
            &sql,
            params_from_iter(dyn_params.iter().map(|b| b.as_ref())),
        )?;

        println!(
            "{}",
            format!(
                "Bike - id:{} set to: '{} {} {}'",
                &bike.bike_id, &bike.code, &bike.name, &bike.added
            )
            .blue()
        );
    }

    Ok(())
}

fn buy(conn: &mut Connection, command: Command) -> Result<()> {
    let result: BuyResult = helpers::get::buy(conn, command)?;

    let buys: Vec<BuyList> = if let helpers::BuyResult::List(buys) = result {
        buys
    } else {
        unreachable!()
    };

    for buy_def in buys {
        let mut category_id: Option<i32> = None;
        let mut bike_id: Option<i32> = None;
        let mut is_changed: bool = false;

        let buy: BuyList = helpers::editor::edit_buy(buy_def.clone()).expect("failed to edit buy");

        let tags_str: String = buy
            .tags
            .split(", ")
            .map(|s| {
                let mut t = String::from(s);
                t.insert(0, '+');
                t
            })
            .collect::<Vec<String>>()
            .join(", ");

        if buy.target != buy_def.target {
            is_changed = true;
            let target_code: Vec<String> = buy
                .target
                .clone()
                .split(":")
                .map(|s| s.to_string())
                .collect();

            if target_code.len() > 2 || (target_code.len() == 2 && target_code[0].is_empty()) {
                warn!(format!(
                    "Incorrect target code format. \nExpected `[abbr]:[int]`, but given - {}",
                    &buy.target
                ));
                continue;
            }

            let abbr: &str = target_code[0].as_str();

            if !target_code[0].is_empty() {
                let Some(category) = get_category(conn, abbr)? else {
                    warn!(format!("category - '{}' does not exist.", abbr));
                    continue;
                };
                category_id = Some(category.id)
            };

            if target_code.len() > 1 && !target_code[1].is_empty() {
                let parsed_id: Option<u8> = target_code[1].parse().ok();

                let Some(id) = parsed_id else {
                    warn!(format!(
                        "Incorrect target code format. \nExpected `[abbr]:[int]`, but given - {}",
                        &buy.target));
                    continue;
                };

                let Some(bike) = get_bike(conn, abbr, id)? else {
                    warn!(format!("bike - '{}:{}' does not exist.", &abbr, &id));
                    continue;
                };

                bike_id = Some(bike.id);
            };
        }

        let tx = conn.transaction()?;

        if is_changed {
            if let Some(b_id) = bike_id {
                if let Ok(btb_id) = tx.query_row(
                    "SELECT id FROM buy_to_bike WHERE buy_id = ?1",
                    params![buy.self_id],
                    |row| row.get::<_, i32>(0),
                ) {
                    tx.execute(
                        "UPDATE buy_to_bike
                        SET bike_id = ?1
                        WHERE id = ?2",
                        params![b_id, btb_id],
                    )?;
                } else {
                    tx.execute(
                        "INSERT INTO buy_to_bike (buy_id, bike_id) VALUES (?1, ?2)",
                        params![buy.self_id, b_id],
                    )?;
                }
            } else {
                tx.execute(
                    "DELETE FROM buy_to_bike WHERE buy_id = ?1",
                    params![buy.self_id],
                )?;
            };

            if let Some(c_id) = category_id {
                if let Ok(btc_id) = tx.query_row(
                    "SELECT id FROM buy_to_category WHERE buy_id = ?1",
                    params![buy.self_id],
                    |row| row.get::<_, i32>(0),
                ) {
                    tx.execute(
                        "UPDATE buy_to_category
                        SET category_id = ?1
                        WHERE id = ?2",
                        params![c_id, btc_id],
                    )?;
                } else {
                    tx.execute(
                        "INSERT INTO buy_to_category (buy_id, category_id) VALUES (?1, ?2)",
                        params![buy.self_id, c_id],
                    )?;
                }
            } else {
                tx.execute(
                    "DELETE FROM buy_to_category WHERE buy_id = ?1",
                    params![buy.self_id],
                )?;
            }
        }

        tx.execute(
            "UPDATE buy
            SET
                name = ?1,
                price = ?2,
                datestamp = ?3
            WHERE id = ?4",
            params![buy.name, buy.price, buy.date, buy.self_id],
        )?;

        if buy.tags != buy_def.tags {
            let tags_to_add: HashSet<String> = helpers::tags_diff(&buy.tags, &buy_def.tags);
            let tags_to_del: HashSet<String> = helpers::tags_diff(&buy_def.tags, &buy.tags);

            if !tags_to_add.is_empty() {
                for tag_name in tags_to_add {
                    match tag_name.as_str()  {
                        s if regex_is_match!(r"^\w+$", s) => {
                            let tag_id = tag_get_or_create_tx(&tx, tag_name.as_str())?;
                            tx.execute(
                                "INSERT INTO tag_to_buy (tag_id, buy_id) VALUES (?1, ?2)",
                                params![tag_id, buy.self_id])?;
                    },
                    _ => {
                        warn!(format!("Incorrect tag: '{}'. Skepped.", &tag_name));
                        continue;
                    }
                }
            }
            }

            if !tags_to_del.is_empty() {
                let mut tag_id_query: Vec<String> = vec![];
                for tag_name in &tags_to_del {
                    tag_id_query.push(format!(
                        "tag_id = (SELECT t.id FROM tag t WHERE t.name = '{}')",
                        &tag_name
                    ));
                }

                tx.execute(
                    "DELETE FROM tag_to_buy WHERE buy_id = ?1 AND ?2",
                    params![buy.self_id, format!("({})", tag_id_query.join(" OR "))],
                )?;
            }
        }

        tx.commit()?;

        let deleted_tags: Vec<String> = delete_unused_tags(conn)?;
        println!("DELETED_TAGS: {:?}", &deleted_tags);

        println!(
            "{}",
            format!(
                "Buy - id:\"{0}\" modified to {1} {2} \"{3}\" {4} {5}",
                buy.self_id, buy.target, &tags_str, buy.name, buy.price, buy.date,
            )
            .blue()
        );
        if !deleted_tags.is_empty() {
            println!(
                "{}",
                format!("Deleted tags: {}", deleted_tags.join(", "),).blue()
            );
        }
    }

    Ok(())
}

fn category(conn: &Connection, command: Command) -> Result<()> {
    let categories: Vec<Category> = helpers::get::categories_with_id(conn, command)?;

    for mut category in categories {
        category = helpers::editor::edit_cat(category).expect("failed to edit cat");

        let result = conn.execute(
            "UPDATE category
            SET
                abbr = ?1,
                name = ?2
            WHERE id = ?3
            ",
            params![category.abbr, category.name, category.id],
        );

        match result {
            Ok(_) => {
                println!(
                    "{}",
                    format!(
                        "Category - \"id:{}\" modified to {}: \"{}\"",
                        &category.id, &category.abbr, &category.name,
                    )
                    .blue()
                );
            }
            Err(e) => match e {
                Error::SqliteFailure(err, Some(msg))
                    if err.code == ErrorCode::ConstraintViolation =>
                {
                    if msg.contains("category.abbr") {
                        warn!(format!(
                            "ID: {} - skepped. Category '{}' already exists.",
                            &category.id, &category.abbr
                        ));
                    } else if msg.contains("category.name") {
                        warn!(format!(
                            "ID: {} - skepped. Category '{}' already exists.",
                            &category.id, &category.name
                        ));
                    } else {
                        err_exit!(msg);
                    }
                }
                other => {
                    err_exit!(other);
                }
            },
        }
    }

    Ok(())
}

fn chain_lub(conn: &Connection, command: Command) -> Result<()> {
    let lubs: Vec<ChainLubricationList> = helpers::get::chain_lub(conn, command)?;

    for lub_def in lubs {

        let lub: ChainLubricationList =
            helpers::editor::edit_lub(lub_def.clone()).expect("failed to edit lub");

        let annotation: String = if !lub.annotation.is_empty() {
            format!("\"{}\"", &lub.annotation)
        } else {
            String::new()
        };

        let mut sql: String = "
            UPDATE chain_lubrication
            SET
                datestamp = ?1,
                distance = ?2,
                annotation = ?3
        "
        .to_string();
        let mut dyn_params: Vec<Box<dyn ToSql>> = vec![
            Box::new(lub.date),
            Box::new(lub.passed),
            Box::new(lub.annotation),
        ];

        if lub.bike != lub_def.bike {
            let bike_code: Vec<String> = lub.bike.clone().split(":").map(|s| s.to_string()).collect();
            let abbr: &str = bike_code[0].as_str();
            let Ok(id_in_cat) = bike_code[1].parse() else {
                warn!(format!(
                    "Incorrect bike code format. \nExpected `[abbr]:[int]`, but given - {}",
                    &lub.bike
                ));
                continue;
            };

            let Some(bike) = get_bike(conn, abbr, id_in_cat)? else {
                warn!(format!("bike - '{}:{}' does not exist.", &abbr, &id_in_cat));
                continue;
            };

            let bike_id: i32 = bike.id;

            sql.push_str(format!(", bike_id = ?{}", dyn_params.len() + 1).as_str());
            dyn_params.push(Box::new(bike_id));
        }

        sql.push_str(format!(" WHERE id = ?{}", dyn_params.len() + 1).as_str());
        dyn_params.push(Box::new(lub.lub_id));

        conn.execute(
            &sql,
            params_from_iter(dyn_params.iter().map(|b| b.as_ref())),
        )?;

        println!(
            "{}",
            format!(
                "Chain Lubrication - id:\"{0}\" modified to {1} {2} {3}km {4}",
                lub.lub_id, lub.bike, lub.date, lub.passed, &annotation,
            )
            .blue(),
        );
    }

    Ok(())
}

fn ride(conn: &mut Connection, command: Command) -> Result<()> {
    let result: RideResult = helpers::get::ride(conn, command)?;

    let rides: Vec<RideList> = if let helpers::RideResult::List(rides) = result {
        rides
    } else {
        unreachable!()
    };

    for ride_def in rides {

        let ride: RideList = helpers::editor::edit_ride(ride_def.clone()).expect("failed to edit ride");

        let annotation: String = if !ride.annotation.is_empty() {
            format!("\"{}\"", &ride.annotation)
        } else {
            String::new()
        };

        let tags_str: String = ride
            .tags
            .split(", ")
            .map(|s| {
                let mut t = String::from(s);
                if !t.is_empty() {
                    t.insert(0, '+');
                }    
                t
            })
            .collect::<Vec<String>>()
            .join(", ");

        let mut sql: String = "
            UPDATE ride
            SET
                datestamp = ?1,
                distance = ?2,
                annotation = ?3
        "
        .to_string();
        let mut dyn_params: Vec<Box<dyn ToSql>> = vec![
            Box::new(ride.date),
            Box::new(ride.distance),
            Box::new(&ride.annotation),
        ];

        if ride.bike != ride_def.bike {
            let bike_code: Vec<String> = ride
                .bike
                .clone()
                .split(":")
                .map(|s| s.to_string())
                .collect();
            let abbr: &str = bike_code[0].as_str();
            let id_in_cat: Option<u8> = bike_code[1].parse().ok();

            let Some(id) = id_in_cat else {
                warn!(
                    format!(
                        "Incorrect bike code format. Skepped. \nExpected `[abbr]:[int]`, but given - {}.", 
                        &ride.bike
                        )
                    );
                continue;
            };

            let Some(bike) = get_bike(conn, abbr, id)? else {
                warn!(format!("bike - '{}:{}' does not exist.", &abbr, &id));
                continue;
            };

            let bike_id: i32 = bike.id;

            sql.push_str(format!(", bike_id = ?{}", dyn_params.len() + 1).as_str());
            dyn_params.push(Box::new(bike_id));
        }

        sql.push_str(format!(" WHERE id = ?{}", dyn_params.len() + 1).as_str());
        dyn_params.push(Box::new(ride.ride_id));

        let tx = conn.transaction()?;

        tx.execute(
            &sql,
            params_from_iter(dyn_params.iter().map(|b| b.as_ref())),
        )?;

        if ride.tags != ride_def.tags {
            let tags_to_add: HashSet<String> = helpers::tags_diff(&ride.tags, &ride_def.tags);
            // err_exit!(format!("TG_TO_ADD: {:?}", &tags_to_add));
            let tags_to_del: HashSet<String> = helpers::tags_diff(&ride_def.tags, &ride.tags);
            println!("TG_TO_DEL: {:?}", &tags_to_del);

            if !tags_to_add.is_empty() {
                for tag_name in tags_to_add {
                    let tag_id = tag_get_or_create_tx(&tx, tag_name.as_str())?;
                    tx.execute(
                        "INSERT INTO tag_to_ride (tag_id, ride_id) VALUES (?1, ?2)",
                        params![tag_id, ride.ride_id],
                    )?;
                }
            }

            if !tags_to_del.is_empty() {
                let mut tag_id_query: Vec<String> = vec![];
                for tag_name in &tags_to_del {
                    tag_id_query.push(format!(
                        "tag_id = (SELECT id FROM tag WHERE name = '{}')",
                        &tag_name
                    ));
                }
                println!("TAGS: {:?}", &tag_id_query);

                tx.execute(
                    "DELETE FROM tag_to_ride WHERE ride_id = ?1 AND ?2",
                    params![ride.id, format!("({})", tag_id_query.join(" OR "))],
                )?;
            }
        }

        tx.commit()?;

        let deleted_tags: Vec<String> = delete_unused_tags(conn)?;

        println!(
            "{}",
            format!(
                "Ride - id:\"{0}\" modified to {1} {2} {3} {4} {5}",
                ride.ride_id, ride.bike, ride.date, ride.distance, &tags_str, &annotation,
            )
            .blue()
        );
        if !deleted_tags.is_empty() {
            println!(
                "{}",
                format!("Deleted tags: {}", deleted_tags.join(", "),).blue()
            );
        }
    }

    Ok(())
}
