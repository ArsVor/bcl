use chrono::NaiveDate;
use rusqlite::{Connection, Result, ToSql, params_from_iter};
use std::collections::HashSet;

use crate::cli::structs::{Command, Date, Field};
use crate::db::models::{BikeList, BuyList, Category, ChainLubricationList, RideList};
use crate::db::queries::{get_category, get_included_excluded};

pub fn get_date(date_parts: Date) -> Option<NaiveDate> {
    if date_parts.day.is_some() {
        Some(date_parts.to_naive())
    } else if date_parts.month.is_some() {
        Some(
            NaiveDate::from_ymd_opt(
                date_parts.year_or_now(),
                date_parts.month.unwrap(),
                date_parts.day.unwrap_or(1),
            )
            .unwrap(),
        )
    } else if date_parts.year.is_some() {
        Some(
            NaiveDate::from_ymd_opt(
                date_parts.year.unwrap(),
                date_parts.month.unwrap_or(1),
                date_parts.day.unwrap_or(1),
            )
            .unwrap(),
        )
    } else {
        None
    }
}

pub mod get {
    use super::*;

    pub fn categories(conn: &Connection) -> Result<Vec<Category>> {
        let mut stmt = conn.prepare("SELECT * FROM category")?;
        let category_iter = stmt.query_map([], Category::from_row)?;

        let mut categories: Vec<Category> = Vec::new();

        for category in category_iter {
            categories.push(category?);
        }

        Ok(categories)
    }

    pub fn tag(conn: &Connection) -> Result<Vec<String>> {
        let mut stmt = conn.prepare("SELECT name FROM tag")?;
        let tag_iter = stmt.query_map([], |row| row.get::<_, String>(0))?;

        let mut tags: Vec<String> = Vec::new();

        for tag in tag_iter {
            tags.push(tag?);
        }

        Ok(tags)
    }

    pub fn bike(conn: &Connection, command: Command) -> Result<Vec<BikeList>> {
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
            stmt.query_map([cat.id], BikeList::from_row)?
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
            stmt.query_map([], BikeList::from_row)?
                .collect::<Result<Vec<_>, _>>()?
        };

        Ok(bikes)
    }

    pub fn buy(conn: &Connection, command: Command) -> Result<Vec<BuyList>> {
        let mut select_sql: String = "
            SELECT 
                b.id AS buy_id,
                b.name,
                b.price,
                b.datestamp,
                COALESCE(GROUP_CONCAT(t.name, ', '), '') AS tags,
            CASE 
                WHEN c.abbr IS NULL AND bk.id_in_cat IS NULL 
                    THEN '' 
                ELSE concat(
                    c.abbr, 
                    ':', 
                    COALESCE(bk.id_in_cat, '')
                ) 
            END AS bike_or_category
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

        if let Some(category) = category {
            where_sql.push(format!("c.abbr = ?{}", where_sql.len() + 1));
            dyn_params.push(Box::new(category));
        }

        if let Some(bike_id) = bike_id {
            where_sql.push(format!("bk.id_in_cat = ?{}", where_sql.len() + 1));
            dyn_params.push(Box::new(bike_id));
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

        select_sql
            .push_str("GROUP BY b.id, b.name, b.price, b.datestamp ORDER BY b.datestamp DESC");
        if command.lim > 0 {
            select_sql.push_str(&format!(" LIMIT {}", &command.lim));
        }

        let mut stmt = conn.prepare(&select_sql)?;
        let buys_iter = stmt.query_map(
            params_from_iter(dyn_params.iter().map(|b| b.as_ref())),
            BuyList::from_row,
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

        Ok(buys)
    }

    pub fn ride(conn: &Connection, command: Command) -> Result<Vec<RideList>> {
        let mut select_sql: String = "
            SELECT
                r.id,
                r.datestamp,
                r.distance,
                concat(c.abbr, ':', b.id_in_cat) as cat_bike,
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

        if let Some(category) = category {
            where_sql.push(format!("c.abbr = ?{}", where_sql.len() + 1));
            dyn_params.push(Box::new(category));
        }

        if let Some(bike_id) = bike_id {
            where_sql.push(format!("b.id_in_cat = ?{}", where_sql.len() + 1));
            dyn_params.push(Box::new(bike_id));
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
        select_sql.push_str("GROUP BY r.id ORDER BY r.datestamp DESC");
        if command.lim > 0 {
            select_sql.push_str(&format!(" LIMIT {}", &command.lim));
        }

        let (include_id, exclude_id): (HashSet<i32>, HashSet<i32>) =
            get_included_excluded(conn, command, "ride")?;

        let mut stmt = conn.prepare(&select_sql)?;
        let rides_iter = stmt.query_map(
            params_from_iter(dyn_params.iter().map(|b| b.as_ref())),
            RideList::from_row,
        )?;

        let mut rides: Vec<RideList> = Vec::new();
        let mut num: i32 = 1;
        for ride_result in rides_iter {
            let mut ride = ride_result?;
            if (include_id.contains(&ride.ride_id) || include_id.is_empty())
                && !exclude_id.contains(&ride.ride_id)
            {
                ride.id = num;
                num += 1;
                rides.push(ride);
            }
        }

        Ok(rides)
    }

    pub fn chain_lub(conn: &Connection, command: Command) -> Result<Vec<ChainLubricationList>> {
        let mut select_sql: String = "
            SELECT
                ROW_NUMBER() OVER (ORDER BY l.datestamp DESC) AS row_num,
                l.id,
                l.datestamp,
                COALESCE(l.annotation, '') AS ann,
                concat(c.abbr, ':', b.id_in_cat)
            FROM chain_lubrication l
            JOIN bike b ON b.id = l.bike_id
            JOIN category c ON c.id = b.category_id
        "
        .to_string();

        let mut where_sql: Vec<String> = vec![];
        let mut dyn_params: Vec<Box<dyn ToSql>> = Vec::new();
        let eq: bool = command.date.day.is_some();

        let date: Option<NaiveDate> = get_date(command.date.clone());
        let date_lt: Option<NaiveDate> = get_date(command.lt.clone());
        let date_gt: Option<NaiveDate> = get_date(command.gt.clone());

        let bike_id: Option<u8> = command.bike_id.get();
        let category: Option<String> = command.category.get();

        if let Some(date) = date {
            match eq {
                true => where_sql.push(format!("l.datestamp = ?{}", where_sql.len() + 1)),
                false => where_sql.push(format!("l.datestamp >= ?{}", where_sql.len() + 1)),
            }
            dyn_params.push(Box::new(date));
        } else {
            if let Some(date_gt) = date_gt {
                where_sql.push(format!("l.datestamp > ?{}", where_sql.len() + 1));
                dyn_params.push(Box::new(date_gt));
            }
            if let Some(date_lt) = date_lt {
                where_sql.push(format!("l.datestamp < ?{}", where_sql.len() + 1));
                dyn_params.push(Box::new(date_lt));
            }
        }

        if let Some(category) = category {
            where_sql.push(format!("c.abbr = ?{}", where_sql.len() + 1));
            dyn_params.push(Box::new(category));
        }

        if let Some(bike_id) = bike_id {
            where_sql.push(format!("b.id_in_cat = ?{}", where_sql.len() + 1));
            dyn_params.push(Box::new(bike_id));
        }

        if !command.annotation.is_empty() {
            let name: String = command.annotation.join(" ");
            where_sql.push(format!("l.annotation LIKE ?{}", where_sql.len() + 1));
            dyn_params.push(Box::new(format!("%{}%", &name)));
        }

        if !where_sql.is_empty() {
            select_sql.push_str(" WHERE ");
            select_sql.push_str(&where_sql.join(" AND "));
        }
        select_sql.push_str("GROUP BY l.id ORDER BY l.datestamp DESC");
        if command.lim > 0 {
            select_sql.push_str(&format!(" LIMIT {}", &command.lim));
        }

        let mut stmt = conn.prepare(&select_sql)?;
        let lubs_iter = stmt.query_map(
            params_from_iter(dyn_params.iter().map(|b| b.as_ref())),
            ChainLubricationList::from_row,
        )?;

        let mut lubs: Vec<ChainLubricationList> = Vec::new();
        for lub in lubs_iter {
            lubs.push(lub?);
        }

        Ok(lubs)
    }
}
