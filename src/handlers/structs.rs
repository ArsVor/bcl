use std::collections::HashMap;

use chrono::NaiveDate;

use crate::{
    cli::structs::Command,
    db::models::{BuyInfo, RideInfo},
};

#[derive(Debug, Clone)]
pub struct BuysInfoReport {
    pub buys_count: u32,
    pub date_eq: Option<NaiveDate>,
    pub date_lt: Option<NaiveDate>,
    pub date_gt: Option<NaiveDate>,
    pub target: Option<String>,
    pub iter_type: String,
    pub last_price: f32,
    pub last_date: Option<NaiveDate>,
    pub total_spend: f32,
    pub spend_by_categories: HashMap<String, f32>,
    pub spend_uncategorized: f32,
}

#[derive(Debug, Clone)]
pub struct RidesInfoReport {
    pub rides_count: u32,
    pub date_eq: Option<NaiveDate>,
    pub date_lt: Option<NaiveDate>,
    pub date_gt: Option<NaiveDate>,
    pub target: Option<String>,
    pub iter_type: String,
    pub last_distance: f32,
    pub last_date: Option<NaiveDate>,
    pub total_distance: f32,
    pub distance_by_categories: HashMap<String, f32>,
}

impl BuysInfoReport {
    fn new() -> BuysInfoReport {
        BuysInfoReport {
            buys_count: 0,
            date_eq: None,
            date_lt: None,
            date_gt: None,
            target: None,
            iter_type: String::from("cat"),
            last_price: 0.0,
            last_date: None,
            total_spend: 0.0,
            spend_by_categories: HashMap::new(),
            spend_uncategorized: 0.0,
        }
    }

    pub fn from(buys: Vec<BuyInfo>, command: &Command) -> BuysInfoReport {
        // if buys.is_empty() {
        //     error
        // }

        let mut report: BuysInfoReport = BuysInfoReport::new();

        report.buys_count = buys.len() as u32;

        if command.date.day.is_some() {
            report.date_eq = Some(command.date.to_naive());
        } else if command.date.is_some() {
            (report.date_gt, report.date_lt) = command.date.get_date_range();
        } else {
            if command.gt.is_some() {
                report.date_gt = Some(command.gt.date_or_first());
            }

            if command.lt.is_some() {
                report.date_lt = Some(command.lt.date_or_first());
            }
        }

        if command.bike_id.is_some() {
            report.target = Some(buys[0].bike_name.clone());
            report.iter_type = "".to_string();
        } else if command.category.is_some() {
            report.target = Some(buys[0].category_name.clone());
            report.iter_type = "bike".to_string();
        }

        report.last_price = buys[0].price;
        report.last_date = Some(buys[0].date);

        for buy in buys {
            report.total_spend += buy.price;

            if !report.iter_type.is_empty() {
                let cat: String = match report.iter_type.as_str() {
                    "bike" => buy.bike_name,
                    "cat" => buy.category_name,
                    _ => {
                        unreachable!()
                    }
                };

                if cat.is_empty() {
                    report.spend_uncategorized += buy.price;
                } else if let Some(val) = report.spend_by_categories.get_mut(&cat) {
                    *val += buy.price;
                } else {
                    report.spend_by_categories.insert(cat, buy.price);
                }
            }
        }

        report
    }
}

impl RidesInfoReport {
    fn new() -> RidesInfoReport {
        RidesInfoReport {
            rides_count: 0,
            date_eq: None,
            date_lt: None,
            date_gt: None,
            target: None,
            iter_type: String::from("cat"),
            last_distance: 0.0,
            last_date: None,
            total_distance: 0.0,
            distance_by_categories: HashMap::new(),
        }
    }

    pub fn from(rides: Vec<RideInfo>, command: &Command) -> RidesInfoReport {
        let mut report: RidesInfoReport = RidesInfoReport::new();

        report.rides_count = rides.len() as u32;

        if command.date.day.is_some() {
            report.date_eq = Some(command.date.to_naive());
        } else if command.date.is_some() {
            (report.date_gt, report.date_lt) = command.date.get_date_range();
        } else {
            if command.gt.is_some() {
                report.date_gt = Some(command.gt.date_or_first());
            }

            if command.lt.is_some() {
                report.date_lt = Some(command.lt.date_or_first());
            }
        }

        if command.bike_id.is_some() {
            report.target = Some(rides[0].bike.clone());
            report.iter_type = "".to_string();
        } else if command.category.is_some() {
            report.target = Some(rides[0].category.clone());
            report.iter_type = "bike".to_string();
        }

        report.last_distance = rides[0].distance;
        report.last_date = Some(rides[0].date);

        for ride in rides {
            report.total_distance += ride.distance;

            if !report.iter_type.is_empty() {
                let cat: String = match report.iter_type.as_str() {
                    "bike" => ride.bike,
                    "cat" => ride.category,
                    _ => {
                        unreachable!()
                    }
                };

                if let Some(val) = report.distance_by_categories.get_mut(&cat) {
                    *val += ride.distance;
                } else {
                    report.distance_by_categories.insert(cat, ride.distance);
                }
            }
        }

        report
    }
}
