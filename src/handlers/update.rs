use owo_colors::OwoColorize;
use rusqlite::{Connection, Result, params};

use crate::cli::structs::Command;
use crate::db::models::{Bike, Buy, Category, ChainLubrication, Ride};
use crate::db::queries::{get_bike, get_category, tag_del_if_unused, tag_get_or_create_tx};
use crate::{err_exit, suc_exit};

pub fn rote(mut conn: Connection, command: Command) -> Result<()> {
    let obj = command.object.unwrap();

    let _ = match obj.as_str() != "tag" {
        true => {
            let id = if let Some(id) = command.real_id.get() {
                id
            } else {
                err_exit!(
                    "Command params missed.\nExpected: `bcl [dynamic id]/id:[static id]` mod [PARAMS]"
                );
            };
            match obj.as_str() {
                "bike" => bike(&conn, command, id),
                "buy" => buy(&mut conn, command, id),
                "cat" => cat(&conn, command, id),
                "lub" => lub(&conn, command, id),
                "ride" => ride(&mut conn, command, id),
                _ => Ok(()),
            }
        }
        false => tag(&mut conn, command),
    };
    Ok(())
}

fn bike(conn: &Connection, command: Command, id: u32) -> Result<()> {
    if command.category.is_none() && command.annotation.is_empty() && command.date.is_none() {
        suc_exit!("Nothing to do!");
    }

    let mut bike = conn.query_row(
        "SELECT * 
        FROM bike
        WHERE id = ?1",
        params![id],
        Bike::from_row,
    )?;

    let mut category_abbr: String = conn.query_row(
        "SELECT abbr
        FROM category
        WHERE id = ?1",
        params![bike.category_id],
        |row| row.get(0),
    )?;

    if command.category.is_some() && command.category.unwrap() != category_abbr {
        let cat: Category = get_category(conn, command.category.unwrap().as_str()).unwrap();
        category_abbr = cat.abbr;

        let id_in_cat: i32 = conn.query_row(
            "SELECT id_in_cat
            FROM bike
            WHERE category_id = ?1
            ORDER BY id DESC
            LIMIT 1",
            rusqlite::params![cat.id],
            |row| row.get(0),
        )?;
        bike.id_in_cat = id_in_cat + 1;
        bike.category_id = cat.id;
    }

    if !command.annotation.is_empty() {
        bike.name = command.annotation.join(" ");
    }

    if command.date.is_some() {
        bike.datestamp = command.date.to_naive();
    }

    conn.execute(
        "UPDATE bike
        SET 
            category_id = ?1, 
            id_in_cat = ?2,
            name = ?3,
            datestamp = ?4
        WHERE id = ?5",
        params![
            bike.category_id,
            bike.id_in_cat,
            bike.name,
            bike.datestamp,
            bike.id
        ],
    )?;

    println!(
        "{}",
        format!(
            "Bike id:\"{0}\" modified to: {1}:{2} {3} {4}",
            &bike.id, &category_abbr, &bike.id_in_cat, &bike.name, &bike.datestamp,
        )
        .blue()
    );

    Ok(())
}

fn buy(conn: &mut Connection, command: Command, id: u32) -> Result<()> {
    if command.val.is_none()
        && command.date.is_none()
        && command.category.is_none()
        && command.bike_id.is_none()
        && command.include_tags.is_empty()
        && command.exclude_tags.is_empty()
        && command.annotation.is_empty()
    {
        suc_exit!("Nothing to do!");
    }

    let mut cat: Option<Category> = None;
    let mut bike: Option<Bike> = None;
    let mut tags_to_check: Vec<String> = Vec::new();
    let mut deleted_tags: Vec<String> = Vec::new();
    let mut target: String = String::new();

    let mut buy: Buy = conn.query_row(
        "SELECT
            b.id AS buy_id,
            b.name,
            b.price,
            b.datestamp,
            COALESCE(GROUP_CONCAT(t.name, ', '), '') AS tags
        FROM buy b
        LEFT JOIN tag_to_buy tb ON tb.buy_id = b.id
        LEFT JOIN tag t ON t.id = tb.tag_id
        WHERE b.id = ?1
        GROUP BY b.id",
        params![id],
        Buy::from_row,
    )?;

    if !command.annotation.is_empty() {
        buy.name = command.annotation.join(" ");
    }

    if let Some(val) = command.val.get() {
        buy.price = val
    }

    if command.date.is_some() {
        buy.datestamp = command.date.to_naive()
    }

    if let Some(category) = command.category.get() {
        let abbr: &str = category.as_str();
        target.push_str(abbr);
        target.push(':');
        cat = Some(get_category(conn, abbr)?);

        if let Some(bike_id) = command.bike_id.get() {
            target.push_str(&bike_id.to_string());
            bike = Some(get_bike(conn, abbr, bike_id)?);
        }
    }

    let tx = conn.transaction()?;

    tx.execute(
        "UPDATE buy
        SET
            name = ?1,
            price = ?2,
            datestamp = ?3
        WHERE id = ?4",
        params![buy.name, buy.price, buy.datestamp, buy.id],
    )?;

    if let Some(bike_obj) = bike {
        if let Ok(btb_id) = tx.query_row(
            "SELECT id FROM buy_to_bike WHERE buy_id = ?1",
            params![buy.id],
            |row| row.get::<_, i32>(0),
        ) {
            tx.execute(
                "UPDATE buy_to_bike
                SET bike_id = ?1
                WHERE id = ?2",
                params![bike_obj.id, btb_id],
            )?;
        } else {
            tx.execute(
                "INSERT INTO buy_to_bike (buy_id, bike_id) VALUES (?1, ?2)",
                params![buy.id, bike_obj.id],
            )?;
        }
    }

    if let Some(category) = cat {
        if let Ok(btc_id) = tx.query_row(
            "SELECT id FROM buy_to_category WHERE buy_id = ?1",
            params![buy.id],
            |row| row.get::<_, i32>(0),
        ) {
            tx.execute(
                "UPDATE buy_to_category
                SET category_id = ?1
                WHERE id = ?2",
                params![category.id, btc_id],
            )?;
        } else {
            tx.execute(
                "INSERT INTO buy_to_category (buy_id, category_id) VALUES (?1, ?2)",
                params![buy.id, category.id],
            )?;
        }
    }

    if !command.include_tags.is_empty() {
        let tags: Vec<&str> = buy.tags.split(", ").collect();
        for tag_name in command.include_tags {
            if !tags.contains(&tag_name.as_str()) {
                let tag_id = tag_get_or_create_tx(&tx, tag_name.as_str())?;
                tx.execute(
                    "INSERT INTO tag_to_buy (tag_id, buy_id) VALUES (?1, ?2)",
                    params![tag_id, buy.id],
                )?;
            }
        }
    }

    if !command.exclude_tags.is_empty() {
        for tag_name in command.exclude_tags {
            if let Ok(tag_id) = tx.query_row(
                "SELECT id FROM tag WHERE name = ?1",
                params![tag_name],
                |row| row.get::<_, i32>(0),
            ) {
                tx.execute(
                    "DELETE FROM tag_to_buy WHERE tag_id = ?1 AND buy_id = ?2",
                    params![tag_id, buy.id],
                )?;
                tags_to_check.push(tag_name);
            }
        }
    }

    tx.commit()?;

    if !tags_to_check.is_empty() {
        for tag_name in tags_to_check {
            if let Some(tag_name) = tag_del_if_unused(conn, tag_name.as_str())? {
                deleted_tags.push(tag_name);
            }
        }
    }

    println!(
        "{}",
        format!(
            "Buy id:\"{0}\" modified to {1} {2} {3} {4} {5}",
            buy.id, &target, buy.tags, buy.name, buy.price, buy.datestamp,
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

fn cat(conn: &Connection, command: Command, id: u32) -> Result<()> {
    if command.annotation.is_empty() && command.category.is_none() {
        suc_exit!("Nothing to do!");
    }

    let mut category = conn.query_row(
        "SELECT *
        FROM category
        WHERE id = ?1",
        params![id],
        Category::from_row,
    )?;

    if let Some(abbr) = command.category.get() {
        category.abbr = abbr
    };

    if !command.annotation.is_empty() {
        category.name = command.annotation.join(" ");
    }

    conn.execute(
        "UPDATE category
        SET 
            abbr = ?1,
            name = ?2
        WHERE id = ?3",
        params![category.abbr, category.name, category.id],
    )?;

    println!(
        "{}",
        format!(
            "Category id:\"{0}\" modified to: {1} - {2}",
            &category.id, &category.abbr, &category.name,
        )
        .blue()
    );

    Ok(())
}

fn lub(conn: &Connection, command: Command, id: u32) -> Result<()> {
    if command.bike_id.is_none() ^ command.category.is_none() {
        err_exit!(format!(
            "expected [category]:[id] but given {0}:{1}",
            command.category.unwrap_or(" ".to_string()),
            command
                .bike_id
                .get()
                .map(|id| id.to_string())
                .unwrap_or_else(|| " ".to_string())
        ));
    }

    if command.bike_id.is_none() && command.annotation.is_empty() && command.date.is_none() {
        suc_exit!("Nothing to do!");
    }

    let mut bike_abbr: String = String::new();

    let mut lub = conn.query_row(
        "SELECT *
        FROM chain_lubrication
        WHERE id = ?1",
        params![id],
        ChainLubrication::from_row,
    )?;

    if !command.annotation.is_empty() {
        lub.annotation = command.annotation.join(" ");
    }

    if command.date.is_some() {
        lub.datestamp = command.date.to_naive();
    }

    if command.bike_id.is_some() {
        let bike = get_bike(
            conn,
            command.category.unwrap().as_str(),
            command.bike_id.unwrap(),
        )?;
        lub.bike_id = bike.id;

        bike_abbr = format!("{}:{}", command.category.unwrap(), command.bike_id.unwrap());
    }

    if let Some(val) = command.val.get() {
        lub.distance = val;
    }

    conn.execute(
        "UPDATE chain_lubrication
        SET
            bike_id = ?1,
            datestamp = ?2,
            distance = ?3,
            annotation = ?4
        WHERE id = ?5",
        params![lub.bike_id, lub.datestamp, lub.distance, lub.annotation, lub.id],
    )?;

    println!(
        "{}",
        format!(
            "Chain Lubrication id:\"{0}\" modified to: {1} {2} {3}km {4}",
            &lub.id, &bike_abbr, &lub.datestamp, lub.distance, &lub.annotation
        )
        .blue()
    );

    Ok(())
}

fn ride(conn: &mut Connection, command: Command, id: u32) -> Result<()> {
    if command.val.is_none()
        && command.date.is_none()
        && command.category.is_none()
        && command.bike_id.is_none()
        && command.include_tags.is_empty()
        && command.exclude_tags.is_empty()
        && command.annotation.is_empty()
    {
        suc_exit!("Nothing to do!");
    }

    if command.category.is_some() && command.bike_id.is_none() {
        err_exit!(format!(
            "Params missed expected: '[category]:[bike_id]', but given '{}:'.",
            &command.category.unwrap()
        ));
    }

    let mut tags_to_check: Vec<String> = Vec::new();
    let mut deleted_tags: Vec<String> = Vec::new();

    let mut ride: Ride = conn.query_row(
        "SELECT
            r.id AS ride_id,
            r.bike_id,
            r.datestamp,
            r.distance,
            c.abbr,
            b.id_in_cat,
            COALESCE(r.annotation, '') AS ann,
            COALESCE(GROUP_CONCAT(t.name, ', '), '') AS tags
        FROM ride r
        JOIN bike b ON b.id = r.bike_id
        JOIN category c ON c.id = b.category_id
        LEFT JOIN tag_to_ride tr ON tr.ride_id = r.id
        LEFT JOIN tag t ON t.id = tr.tag_id
        WHERE r.id = ?1
        GROUP BY r.id",
        params![id],
        Ride::from_row,
    )?;

    let mut tags: Vec<String> = ride.tags.split(", ").map(|s| s.to_string()).collect();

    if !command.annotation.is_empty() {
        ride.annotation = command.annotation.join(" ");
    }

    if command.date.is_some() {
        ride.datestamp = command.date.to_naive();
    }

    if let Some(distance) = command.val.get() {
        ride.distance = distance;
    }

    if let (Some(abbr), Some(bike_id)) = (command.category.get(), command.bike_id.get()) {
        let bike: Bike = get_bike(conn, abbr.as_str(), bike_id)?;
        ride.bike_id = bike.id;
        ride.abbr = abbr;
        ride.id_in_cat = bike_id;
    }

    let tx = conn.transaction()?;

    tx.execute(
        "UPDATE ride
        SET
            bike_id = ?1,
            datestamp = ?2,
            distance = ?3,
            annotation = ?4
        WHERE id = ?5
        ",
        params![
            ride.bike_id,
            ride.datestamp,
            ride.distance,
            ride.annotation,
            ride.id
        ],
    )?;

    if !command.include_tags.is_empty() {
        for tag_name in command.include_tags {
            if !tags.contains(&tag_name) {
                tags.push(tag_name.clone());
                let tag_id: i32 = tag_get_or_create_tx(&tx, &tag_name)?;

                tx.execute(
                    "INSERT INTO tag_to_ride (tag_id, ride_id) VALUES (?1, ?2)",
                    params![tag_id, ride.id],
                )?;
            }
        }
    }

    if !command.exclude_tags.is_empty() {
        for tag_name in &command.exclude_tags {
            if let Ok(tag_id) = tx.query_row(
                "SELECT id FROM tag WHERE name = ?1",
                params![tag_name],
                |row| row.get::<_, i32>(0),
            ) {
                tx.execute(
                    "DELETE FROM tag_to_ride WHERE tag_id = ?1 AND ride_id = ?2",
                    params![tag_id, ride.id],
                )?;

                tags_to_check.push(tag_name.clone());
            }
        }
    }

    tx.commit()?;

    if !tags_to_check.is_empty() {
        for tag_name in tags_to_check {
            if let Some(tag_name) = tag_del_if_unused(conn, tag_name.as_str())? {
                deleted_tags.push(tag_name);
            }
        }
    }

    let tags_str = tags
        .into_iter()
        .filter(|t| !command.exclude_tags.contains(t))
        .map(|mut t| {
            t.insert(0, '+');
            t
        })
        .collect::<Vec<_>>()
        .join(", ");

    let mut annotation: String = String::new();
    if !ride.annotation.is_empty() {
        annotation = format!("\"{}\"", ride.annotation);
    }

    println!(
        "{}",
        format!(
            "Ride id:\"{0}\" modified to {1}:{2} {3} {4} {5} {6}",
            ride.id,
            ride.abbr,
            ride.id_in_cat,
            ride.datestamp,
            ride.distance,
            &tags_str,
            &annotation,
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
    if command.annotation.is_empty() && command.include_tags.is_empty() {
        suc_exit!("Nothing to do!");
    }

    let old_tags_iter = command.include_tags.iter();
    let new_tags_iter = command.annotation.iter();
    let tag_pairs = old_tags_iter.zip(new_tags_iter);
    let tx = conn.transaction()?;

    for (old_tag, new_tag) in tag_pairs.clone() {
        let rows_affected = tx
            .execute(
                "UPDATE tag
             SET name = ?2
             WHERE name = ?1",
                params![old_tag, new_tag],
            )
            .unwrap_or(0);

        if rows_affected == 0 {
            err_exit!(format!("tag {} - not found.", old_tag));
        }
    }

    tx.commit()?;

    println!("{}", "Tags modified:".blue());
    tag_pairs.for_each(|t| println!("{}", format!("{}  {}", t.0, t.1).blue()));

    Ok(())
}
