use chrono::NaiveDate;
use owo_colors::OwoColorize;
use rusqlite::{Result, Row};
use std::fmt;
use tabled::Tabled;

#[derive(Debug, Clone)]
pub struct Opt<T>(pub Option<T>);

#[derive(Debug, Clone, Tabled)]
pub struct Category {
    pub id: i32,
    pub abbr: String,
    pub name: String,
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

#[derive(Debug, Clone, Tabled)]
pub struct BikeList {
    pub id: i32,
    pub bike_id: i32,
    pub category: String,
    pub bike: String,
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
    pub annotation: Opt<String>,
}

#[derive(Debug, Clone, Tabled)]
pub struct ChainLubricationList {
    pub id: i32,
    pub lub_id: i32,
    pub bike: String,
    pub datestamp: NaiveDate,
    pub annotation: Opt<String>,
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

impl<T: fmt::Display> fmt::Display for Opt<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(v) => write!(f, "{}", &v),
            None => write!(f, "—"), // тут можна поставити "" або будь-що
        }
    }
}

impl<T> Opt<T> {
    pub fn unwrap(&self) -> T
    where
        T: Clone,
    {
        self.0.clone().unwrap()
    }
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

impl BikeList {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.get("row_num")?,
            bike_id: row.get("id")?,
            category: row.get("abbr")?,
            bike: row.get("name")?,
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
            annotation: Opt(row.get::<_, Option<String>>("annotation")?),
        })
    }
}

impl ChainLubricationList {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            lub_id: row.get(1)?,
            bike: row.get(4)?,
            datestamp: row.get(2)?,
            annotation: Opt(row.get::<_, Option<String>>(3)?),
        })
    }
}
