use chrono::NaiveDate;
use rusqlite::{Connection, Result, ToSql, params_from_iter};
use std::collections::HashSet;
use std::env;
use std::fs::read_to_string;
use std::io::Write;
use std::process;
use tempfile::NamedTempFile;

use crate::cli::structs::{Command, Date, Field};
use crate::db::models::{BikeList, BuyList, Category, ChainLubricationList, RideList};
use crate::db::queries::get_included_excluded;
use crate::err_exit;

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

pub fn tags_diff(s1: &str, s2: &str) -> HashSet<String> {
    let set1: HashSet<String> = s1.split(", ").map(|s| s.to_string()).collect();
    let set2: HashSet<String> = s2.split(", ").map(|s| s.to_string()).collect();
    set1.difference(&set2).map(|s| s.to_string()).collect()
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

    pub fn category_with_params(conn: &Connection, command: Command) -> Result<Category> {
        let mut select_sql: String = "SELECT * FROM category".to_string();
        let mut where_sql: Vec<String> = vec![];
        let mut dyn_params: Vec<Box<dyn ToSql>> = Vec::new();

        if let Some(id) = command.real_id.get().or(command.id.get()) {
            where_sql.push(format!("id =?{}", where_sql.len() + 1));
            dyn_params.push(Box::new(id));
        };

        if let Some(abbr) = command.category.get() {
            where_sql.push(format!("abbr = ?{}", where_sql.len() + 1));
            dyn_params.push(Box::new(abbr));
        };

        if !command.annotation.is_empty() {
            let name: String = command.annotation.join(" ");
            where_sql.push(format!("name LIKE ?{}", where_sql.len() + 1));
            dyn_params.push(Box::new(format!("%{}%", &name)));
        };

        if !where_sql.is_empty() {
            select_sql.push_str(" WHERE ");
            select_sql.push_str(&where_sql.join(" AND "));
        }

        let mut stmt = conn.prepare(&select_sql)?;
        let mut categories: Vec<Category> = stmt
            .query_map(
                params_from_iter(dyn_params.iter().map(|b| b.as_ref())),
                Category::from_row,
            )?
            .collect::<Result<Vec<_>, _>>()?;

        if categories.len() > 1 {
            err_exit!("Not enoughs params. Can't select 1 bike.");
        } else if let Some(category) = categories.pop() {
            Ok(category)
        } else {
            err_exit!("bike category for your request was not found.");
        }
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
        let mut select_sql: String = "
            SELECT 
                ROW_NUMBER() OVER (ORDER BY c.id) AS row_num,
                b.id, 
                concat(c.abbr, ':', b.id_in_cat) AS code, 
                b.name, 
                b.datestamp  
            FROM bike b 
                JOIN category c ON c.id = b.category_id
        "
        .to_string();
        let mut where_sql: Vec<String> = vec![];
        let mut dyn_params: Vec<Box<dyn ToSql>> = Vec::new();

        if let Some(real_id) = command.real_id.get() {
            where_sql.push(format!("b.id = ?{}", where_sql.len() + 1));
            dyn_params.push(Box::new(real_id));
        } else {
            if let Some(category) = command.category.get() {
                where_sql.push(format!("c.abbr = ?{}", where_sql.len() + 1));
                dyn_params.push(Box::new(category));
            }

            if let Some(bike_id) = command.bike_id.get() {
                where_sql.push(format!("b.id_in_cat = ?{}", where_sql.len() + 1));
                dyn_params.push(Box::new(bike_id));
            }

            if !command.annotation.is_empty() {
                let name: String = command.annotation.join(" ");
                where_sql.push(format!("b.name LIKE ?{}", where_sql.len() + 1));
                dyn_params.push(Box::new(format!("%{}%", &name)));
            }
        }

        if !where_sql.is_empty() {
            select_sql.push_str(" WHERE ");
            select_sql.push_str(&where_sql.join(" AND "));
        }

        select_sql.push_str("GROUP BY b.id ORDER BY c.id");
        if command.lim > 0 {
            select_sql.push_str(&format!(" LIMIT {}", &command.lim));
        }

        let mut stmt = conn.prepare(&select_sql)?;
        let bikes: Vec<BikeList> = stmt
            .query_map(
                params_from_iter(dyn_params.iter().map(|b| b.as_ref())),
                BikeList::from_row,
            )?
            .collect::<Result<Vec<_>, _>>()?;

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

        if let Some(real_id) = command.real_id.get() {
            where_sql.push(format!("b.id = ?{}", where_sql.len() + 1));
            dyn_params.push(Box::new(real_id));
        }

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

        if let Some(real_id) = command.real_id.get() {
            where_sql.push(format!("r.id = ?{}", where_sql.len() + 1));
            dyn_params.push(Box::new(real_id));
        }

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

        if let Some(real_id) = command.real_id.get() {
            where_sql.push(format!("l.id = ?{}", where_sql.len() + 1));
            dyn_params.push(Box::new(real_id));
        }

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

pub mod editor {

    use super::*;

    pub fn edit_bike(mut bike: BikeList) -> std::io::Result<BikeList> {
        // 1. Створюємо тимчасовий файл
        let mut tmp = NamedTempFile::new()?;

        // 2. Пишемо дані bike у файл у зручному для редагування форматі
        writeln!(tmp, "id: {}", bike.id)?;
        writeln!(tmp, "bike_id: {}", bike.bike_id)?;
        writeln!(tmp, "code: {}", bike.code)?;
        writeln!(tmp, "name: {}", bike.name)?;
        writeln!(tmp, "date: {}", bike.added)?;

        tmp.flush()?;

        // 3. Визначаємо редактор
        let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

        // 4. Запускаємо редактор
        process::Command::new(editor)
            .arg(tmp.path())
            .status()
            .expect("failed to run editor");

        // 5. Читаємо файл після редагування
        let content = read_to_string(tmp.path())?;

        // 6. Парсимо назад у bikeList (тут простий варіант, можна зробити нормальний парсер)
        for line in content.lines() {
            let mut parts = line.splitn(2, ": ");
            let key = parts.next().unwrap_or("");
            let val = parts.next().unwrap_or("");
            match key {
                "code" => bike.code = val.to_string(),
                "name" => bike.name = val.to_string(),
                "date" => bike.added = val.parse().unwrap_or(bike.added),
                _ => {}
            }
        }

        Ok(bike)
    }

    pub fn edit_buy(mut buy: BuyList) -> std::io::Result<BuyList> {
        let mut tmp = NamedTempFile::new()?;

        writeln!(tmp, "id: {}", buy.id)?;
        writeln!(tmp, "buy_id: {}", buy.buy_id)?;
        writeln!(tmp, "target: {}", buy.target)?;
        writeln!(tmp, "tags: {}", buy.tags)?;
        writeln!(tmp, "name: {}", buy.name)?;
        writeln!(tmp, "price: {}", buy.price)?;
        writeln!(tmp, "date: {}", buy.date)?;

        tmp.flush()?;

        let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

        process::Command::new(editor)
            .arg(tmp.path())
            .status()
            .expect("failed to run editor");

        let content = read_to_string(tmp.path())?;

        for line in content.lines() {
            let mut parts = line.splitn(2, ": ");
            let key = parts.next().unwrap_or("");
            let val = parts.next().unwrap_or("");
            match key {
                "target" => buy.target = val.to_string(),
                "tags" => buy.tags = val.to_string(),
                "name" => buy.name = val.to_string(),
                "price" => buy.price = val.parse().unwrap_or(buy.price),
                "date" => buy.date = val.parse().unwrap_or(buy.date),
                _ => {}
            }
        }

        Ok(buy)
    }

    pub fn edit_cat(mut category: Category) -> std::io::Result<Category> {
        let mut tmp = NamedTempFile::new()?;

        writeln!(tmp, "id: {}", category.id)?;
        writeln!(tmp, "abbr: {}", category.abbr)?;
        writeln!(tmp, "name: {}", category.name)?;

        tmp.flush()?;

        let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

        process::Command::new(editor)
            .arg(tmp.path())
            .status()
            .expect("failed to run editor");

        let content = read_to_string(tmp.path())?;

        for line in content.lines() {
            let mut parts = line.splitn(2, ": ");
            let key = parts.next().unwrap_or("");
            let val = parts.next().unwrap_or("");
            match key {
                "abbr" => if !val.is_empty() {
                   category.abbr =  val.to_string()
                },
                "name" => if !val.is_empty() {
                    category.name = val.to_string()
                },
                _ => {}
            }
        }

        Ok(category)
    }

    pub fn edit_lub(mut lub: ChainLubricationList) -> std::io::Result<ChainLubricationList> {
        let mut tmp = NamedTempFile::new()?;

        writeln!(tmp, "id: {}", lub.lub_id)?;
        writeln!(tmp, "bike: {}", lub.bike)?;
        writeln!(tmp, "date: {}", lub.date)?;
        writeln!(tmp, "annotation: {}", lub.annotation)?;

        tmp.flush()?;

        let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

        process::Command::new(editor)
            .arg(tmp.path())
            .status()
            .expect("failed to run editor");

        let content = read_to_string(tmp.path())?;

        for line in content.lines() {
            let mut parts = line.splitn(2, ": ");
            let key = parts.next().unwrap_or("");
            let val = parts.next().unwrap_or("");
            match key {
                "bike" => lub.bike = val.to_string(),
                "date" => lub.date = val.parse().unwrap_or(lub.date),
                "annotation" => lub.annotation = val.to_string(),
                _ => {}
            }
        }

        Ok(lub)
    }

    pub fn edit_ride(mut ride: RideList) -> std::io::Result<RideList> {
        let mut tmp = NamedTempFile::new()?;

        writeln!(tmp, "id: {}", ride.id)?;
        writeln!(tmp, "ride_id: {}", ride.ride_id)?;
        writeln!(tmp, "bike: {}", ride.bike)?;
        writeln!(tmp, "date: {}", ride.date)?;
        writeln!(tmp, "distance: {}", ride.distance)?;
        writeln!(tmp, "tags: {}", ride.tags)?;
        writeln!(tmp, "annotation: {}", ride.annotation)?;

        tmp.flush()?;

        let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

        process::Command::new(editor)
            .arg(tmp.path())
            .status()
            .expect("failed to run editor");

        let content = read_to_string(tmp.path())?;

        for line in content.lines() {
            let mut parts = line.splitn(2, ": ");
            let key = parts.next().unwrap_or("");
            let val = parts.next().unwrap_or("");
            match key {
                "bike" => ride.bike = val.to_string(),
                "date" => ride.date = val.parse().unwrap_or(ride.date),
                "distance" => ride.distance = val.parse().unwrap_or(ride.distance),
                "tags" => ride.tags = val.to_string(),
                "annotation" => ride.annotation = val.to_string(),
                _ => {}
            }
        }

        Ok(ride)
    }
}
