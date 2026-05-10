use indexmap::IndexMap;

use chrono::{Datelike, NaiveDate};

use crate::{
    cli::structs::Command,
    db::models::{BuyInfo, RideInfo},
};

#[derive(Debug, Default, Clone)]
pub struct FkIds {
    pub category: Option<i32>,
    pub bike: Option<i32>,
}

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
    pub spend_by_categories: IndexMap<String, f32>,
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
    pub distance_by_categories: IndexMap<String, f32>,
}

impl FkIds {
    pub fn new() -> Self {
        Self {
            category: None,
            bike: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.category.is_none() && self.bike.is_none()
    }
}

impl BuysInfoReport {
    fn new() -> BuysInfoReport {
        BuysInfoReport {
            buys_count: 0,
            date_eq: None,
            date_lt: None,
            date_gt: None,
            target: None,
            iter_type: String::from(""),
            last_price: 0.0,
            last_date: None,
            total_spend: 0.0,
            spend_by_categories: IndexMap::new(),
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

        if command.output.is_none() && command.bike_id.is_none() {
            if command.category.is_some() {
                report.target = Some(buys[0].category_name.clone());
                report.iter_type = "bike".to_string();
            } else {
                report.iter_type = "cat".to_string();
            }
        } else if command.bike_id.is_none() {
            if let Some(period) = command.group_by.get() {
                match period.as_ref() {
                    "dayly" => report.iter_type = "dayly".to_string(),
                    "weekly" => report.iter_type = "weekly".to_string(),
                    "monthly" => report.iter_type = "monthly".to_string(),
                    "yearly" => report.iter_type = "yearly".to_string(),
                    _ => {
                        unreachable!()
                    }
                }
            } else if command.category.is_some() {
                report.iter_type = "for bikes".to_string();
            } else {
                report.iter_type = "for categories".to_string();
            }
        }

        if command.bike_id.is_some() {
            report.target = Some(buys[0].bike_name.clone());
        }

        report.last_price = buys.last().unwrap().price;
        report.last_date = Some(buys.last().unwrap().date);

        for buy in buys {
            report.total_spend += buy.price;

            if !report.iter_type.is_empty() {
                let cat: String = match report.iter_type.as_str() {
                    "bike" => buy.bike_name,
                    "cat" => buy.category_name,
                    "for categories" => {
                        if !buy.code.is_empty() {
                            let code: Vec<&str> = buy.code.split(":").collect();
                            format!("{}:", code[0])
                        } else {
                            String::new()
                        }
                    }
                    "for bikes" => {
                        if buy.code.ends_with(":") {
                            String::new()
                        } else {
                            buy.code
                        }
                    }
                    "dayly" => buy.date.format("%y-%m-%d").to_string(),
                    "weekly" => buy.date.iso_week().week().to_string(),
                    "monthly" => buy.date.format("%y-%m").to_string(),
                    "yearly" => buy.date.year().to_string(),
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
            iter_type: String::from(""),
            last_distance: 0.0,
            last_date: None,
            total_distance: 0.0,
            distance_by_categories: IndexMap::new(),
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

        if command.output.is_none() && command.bike_id.is_none() {
            if command.category.is_some() {
                report.target = Some(rides[0].category.clone());
                report.iter_type = "bike".to_string();
            } else {
                report.iter_type = "cat".to_string();
            }
        } else if command.bike_id.is_none() {
            if let Some(period) = command.group_by.get() {
                match period.as_ref() {
                    "dayly" => report.iter_type = "dayly".to_string(),
                    "weekly" => report.iter_type = "weekly".to_string(),
                    "monthly" => report.iter_type = "monthly".to_string(),
                    "yearly" => report.iter_type = "yearly".to_string(),
                    _ => {
                        unreachable!()
                    }
                }
            } else if command.category.is_some() {
                report.iter_type = "by bikes".to_string();
            } else {
                report.iter_type = "by categories".to_string();
            }
        }

        if command.bike_id.is_some() {
            report.target = Some(rides[0].bike.clone());
        }

        report.last_distance = rides.last().unwrap().distance;
        report.last_date = Some(rides.last().unwrap().date);

        for ride in rides {
            report.total_distance += ride.distance;

            if !report.iter_type.is_empty() {
                let cat: String = match report.iter_type.as_str() {
                    "bike" => ride.bike,
                    "cat" => ride.category,
                    "by categories" => {
                        let code: Vec<&str> = ride.code.split(":").collect();
                        format!("{}:", code[0])
                    }
                    "by bikes" => ride.code,
                    "dayly" => ride.date.format("%y-%m-%d").to_string(),
                    "weekly" => ride.date.iso_week().week().to_string(),
                    "monthly" => ride.date.format("%y-%m").to_string(),
                    "yearly" => ride.date.year().to_string(),
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
