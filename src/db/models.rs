use chrono::NaiveDate;
use rusqlite_from_row::FromRow;

#[derive(Debug, FromRow)]
pub struct Category {
    pub id: i32,
    pub abbr: String,
    pub name: String,
}

#[derive(Debug, FromRow)]
pub struct Tag {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, FromRow)]
pub struct Buy {
    pub id: i32,
    pub name: String,
    pub price: f32,
    pub datastamp: NaiveDate,
}

#[derive(Debug, FromRow)]
pub struct Bike {
    pub id: i32,
    pub category_id: i32,
    pub name: String,
    pub datestamp: NaiveDate,
}

#[derive(Debug, FromRow)]
pub struct Ride {
    pub id: i32,
    pub bike_id: i32,
    pub datestamp: NaiveDate,
    pub distance: f32,
    pub annotate: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct ChainLubrication {
    pub id: i32,
    pub bike_id: i32,
    pub datestamp: NaiveDate,
    pub annotate: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct TagToRide {
    pub id: i32,
    pub tag_id: i32,
    pub bike_id: i32,
}

#[derive(Debug, FromRow)]
pub struct TagToBuy {
    pub id: i32,
    pub tag_id: i32,
    pub buy_id: i32,
}

#[derive(Debug, FromRow)]
pub struct BuyToBike {
    pub id: i32,
    pub buy_id: i32,
    pub bike_id: i32,
}

#[derive(Debug, FromRow)]
pub struct BuyToCategory {
    pub id: i32,
    pub buy_id: i32,
    pub category_id: i32,
}
