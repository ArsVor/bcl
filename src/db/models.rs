use chrono::NaiveDate;
use rusqlite::{Result, Row};
// use std::fmt;
use tabled::Tabled;

// #[derive(Debug, Clone)]
// pub struct Opt<T>(pub Option<T>);

#[derive(Debug, Clone, Tabled)]
pub struct Category {
    pub id: i32,
    pub abbr: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct CategoryInfo {
    pub id: i32,
    pub abbr: String,
    pub name: String,
    pub bike_count: u16,
    pub total_spend: f32,
    pub ride_count: u16,
    pub total_distance: f32,
}

#[derive(Debug, Clone, Tabled)]
pub struct Tag {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Tabled)]
pub struct Buy {
    pub id: i32,
    pub name: String,
    pub price: f32,
    pub datestamp: NaiveDate,
    pub tags: String,
}

#[derive(Debug, Clone, Tabled)]
pub struct BuyList {
    pub id: i32,
    pub buy_id: i32,
    pub target: String,
    pub tags: String,
    pub name: String,
    pub price: f32,
    pub date: NaiveDate,
}

#[derive(Debug, Clone, Tabled)]
pub struct Bike {
    pub id: i32,
    pub category_id: i32,
    pub id_in_cat: i32,
    pub name: String,
    pub datestamp: NaiveDate,
}

#[derive(Debug, Clone)]
pub struct BikeInfo {
    pub name: String,
    pub id: i32,
    pub category: String,
    pub code: String,
    pub add_date: NaiveDate,
    pub ride_count: u32,
    pub total_distance: f32,
    pub last_ride: Option<NaiveDate>,
    pub last_distance: f32,
    pub chain_lub: Option<NaiveDate>,
    pub after_lub_distance: f32,
    pub maintenance: Option<NaiveDate>,
    pub total_spend: f32,
}

#[derive(Debug, Clone, Tabled)]
pub struct BikeList {
    pub id: i32,
    pub bike_id: i32,
    pub code: String,
    pub name: String,
    pub added: NaiveDate,
}

#[derive(Debug, Clone, Tabled)]
pub struct Ride {
    pub id: i32,
    pub bike_id: i32,
    pub datestamp: NaiveDate,
    pub distance: f32,
    pub abbr: String,
    pub id_in_cat: u8,
    pub tags: String,
    pub annotation: String,
}

#[derive(Debug, Clone, Tabled)]
pub struct RideList {
    pub id: i32,
    pub ride_id: i32,
    pub bike: String,
    pub date: NaiveDate,
    pub distance: f32,
    pub tags: String,
    pub annotation: String,
}

#[derive(Debug, Clone, Tabled)]
pub struct ChainLubrication {
    pub id: i32,
    pub bike_id: i32,
    pub datestamp: NaiveDate,
    pub distance: f32,
    pub annotation: String,
}

#[derive(Debug, Clone, Tabled)]
pub struct ChainLubricationList {
    pub id: i32,
    pub lub_id: i32,
    pub bike: String,
    pub date: NaiveDate,
    pub passed: f32,
    pub annotation: String,
}

#[derive(Debug, Clone, Tabled)]
pub struct TagToRide {
    pub id: i32,
    pub tag_id: i32,
    pub bike_id: i32,
}

#[derive(Debug, Clone, Tabled)]
pub struct TagToBuy {
    pub id: i32,
    pub tag_id: i32,
    pub buy_id: i32,
}

#[derive(Debug, Clone, Tabled)]
pub struct BuyToBike {
    pub id: i32,
    pub buy_id: i32,
    pub bike_id: i32,
}

#[derive(Debug, Clone, Tabled)]
pub struct BuyToCategory {
    pub id: i32,
    pub buy_id: i32,
    pub category_id: i32,
}

impl Category {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            abbr: row.get("abbr")?,
            name: row.get("name")?,
        })
    }
}

impl CategoryInfo {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.get("cat_id")?,
            abbr: row.get("cat_abbr")?,
            name: row.get("cat_name")?,
            bike_count: row.get("bike_count")?,
            total_spend: row.get("total_spend")?,
            ride_count: row.get("ride_count")?,
            total_distance: row.get("total_distance")?,
        })
    }
}

impl Tag {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            name: row.get("name")?,
        })
    }
}

impl Buy {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            name: row.get(1)?,
            price: row.get(2)?,
            datestamp: row.get(3)?,
            tags: row.get(4)?,
        })
    }
}

impl BuyList {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: 0,
            buy_id: row.get(0)?,
            target: row.get(5)?,
            tags: row.get(4)?,
            name: row.get(1)?,
            price: row.get(2)?,
            date: row.get(3)?,
        })
    }
}

impl Bike {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            category_id: row.get("category_id")?,
            id_in_cat: row.get("id_in_cat")?,
            name: row.get("name")?,
            datestamp: row.get("datestamp")?,
        })
    }
}

impl BikeInfo {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            name: row.get("bike_name")?,
            id: row.get("bike_id")?,
            category: row.get("category_name")?,
            code: row.get("code")?,
            add_date: row.get("add_date")?,
            ride_count: row.get("ride_count")?,
            total_distance: row.get("total_distance")?,
            last_ride: row.get("last_ride")?,
            last_distance: row.get("last_distance")?,
            chain_lub: row.get("chain_lub")?,
            after_lub_distance: row.get("after_lub_distance")?,
            maintenance: row.get("maintenance")?,
            total_spend: row.get("total_spend")?,
        })
    }
}

impl BikeList {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.get("row_num")?,
            bike_id: row.get("id")?,
            code: row.get("code")?,
            name: row.get("name")?,
            added: row.get("datestamp")?,
        })
    }
}

impl Ride {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            bike_id: row.get(1)?,
            datestamp: row.get(2)?,
            distance: row.get(3)?,
            abbr: row.get(4)?,
            id_in_cat: row.get(5)?,
            tags: row.get(7)?,
            annotation: row.get(6)?,
        })
    }
}

impl RideList {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: 0,
            ride_id: row.get(0)?,
            bike: row.get(3)?,
            date: row.get(1)?,
            distance: row.get(2)?,
            tags: row.get(5)?,
            annotation: row.get(4)?,
        })
    }
}

impl ChainLubrication {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            bike_id: row.get("bike_id")?,
            datestamp: row.get("datestamp")?,
            distance: row.get("distance")?,
            annotation: row.get("annotation")?,
        })
    }
}

impl ChainLubricationList {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.get("row_num")?,
            lub_id: row.get("lub_id")?,
            bike: row.get("code")?,
            date: row.get("date")?,
            passed: row.get("dist")?,
            annotation: row.get("ann")?,
        })
    }
}
