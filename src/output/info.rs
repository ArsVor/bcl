use owo_colors::OwoColorize;
use std::cmp;

use crate::{
    db::models::{BikeInfo, BuyInfo, CategoryInfo, ChainLubricationList, RideInfo},
    handlers::structs::{BuysInfoReport, RidesInfoReport},
};

pub fn bike_info(bike: BikeInfo) {
    let width = 17;
    let after_lub_distance: f32 = bike.after_lub_distance;
    let msg: String = format!(
        "Without chain lubrication, passed: {:.2} (km)",
        &after_lub_distance
    );

    println!("{}", format!("\n~~ {} ~~", &bike.name).green());
    println!(
        "{}",
        format!("{:width$} {}", "Category:", &bike.category).green()
    );
    println!("{}", format!("{:width$} {}", "ID:", &bike.id).green());
    println!(
        "{}",
        format!("{:width$} {}", "Bike code:", &bike.code).green()
    );
    println!(
        "{}",
        format!("{:width$} {}", "Added:", &bike.add_date).green()
    );
    println!(
        "{}",
        format!("{:width$} {:.2} (UAH)", "Total spend:", &bike.total_spend).green()
    );
    println!(
        "{}",
        format!("{:width$} {}", "Ride count:", &bike.ride_count).green()
    );
    if let Some(date) = bike.last_ride {
        println!(
            "{}",
            format!(
                "{:width$} {:.2} (km)",
                "Total distance:", &bike.total_distance
            )
            .green()
        );
        println!("{}", format!("{:width$} {}", "Last ride:", &date).green());
        println!(
            "{}",
            format!("{:width$} {:.2} (km)", " distance:", &bike.last_distance).green()
        );
    }
    if let Some(date) = bike.maintenance {
        println!(
            "{}",
            format!("{:width$} {}", "Last maintenance:", &date).green()
        );
    }
    if let Some(date) = bike.chain_lub {
        println!(
            "{}",
            format!("{:width$} {}", "Last chain lub:", &date).green()
        );
    }
    if after_lub_distance > 0.00 {
        if after_lub_distance > 200.00 {
            println!("{}", msg.red());
        } else if after_lub_distance > 150.00 {
            println!("{}", msg.yellow());
        } else {
            println!("{}", msg.green());
        }
    };
}

pub fn buy_info(report: BuysInfoReport) {
    let width: usize = cmp::max(
        report
            .spend_by_categories
            .keys()
            .max_by_key(|k| k.len())
            .unwrap_or(&"".to_string())
            .len()
            + 2,
        15,
    );
    println!("{}", "\n~~ Buys ~~".green());
    if let Some(date) = report.date_eq {
        println!("{}", format!("at:  {}", &date).green());
    } else {
        let mut date_str: String = String::new();

        if let Some(date) = report.date_gt {
            date_str += &format!("from: {}", &date)
        }

        if let Some(date) = report.date_lt {
            date_str += &format!("  to: {}", &date)
        }

        if !date_str.is_empty() {
            println!("{}", date_str.green());
        }
    }
    println!(
        "{}",
        format!("{:width$} {}", "Buys count:", &report.buys_count).green()
    );
    println!(
        "{}",
        format!("{:width$} {:.2} (UAH)", "Last bought:", &report.last_price).green()
    );
    println!(
        "{}",
        format!("{:width$} {}", "on:", &report.last_date.unwrap()).green()
    );
    println!(
        "{}",
        format!("{:width$} {:.2} (UAH)", "Total spend:", &report.total_spend).green()
    );

    match report.iter_type.as_str() {
        "cat" => {
            println!("{}", "\nFor category:".green());
        }
        "bike" => {
            println!("{}", "\nFor bike:".green());
        }
        _ => {}
    }

    if !report.iter_type.is_empty() {
        for (cat, spend) in report.spend_by_categories.iter() {
            println!(
                "{}",
                format!("{:width$} {:.2} (UAH)", format!("{cat}:"), spend).green()
            );
        }
        println!(
            "{}",
            format!(
                "{:width$} {:.2} (UAH)",
                "Uncategorized:", report.spend_uncategorized
            )
            .green()
        )
    }
}

pub fn buy_info_single(buy: &BuyInfo) {
    let width = 6;
    let mut target: &String = &"uncategorized".to_string();

    if !buy.bike_name.is_empty() {
        target = &buy.bike_name;
    } else if !buy.category_name.is_empty() {
        target = &buy.category_name;
    }

    println!("{}", format!("\n~~ {} ~~", &buy.name).green());
    println!("{}", format!("{:width$} {}", "ID:", &buy.buy_id).green());
    println!("{}", format!("{:width$} {}", "Date:", &buy.date).green());
    println!("{}", format!("{:width$} {}", "For:", &target).green());
    println!("{}", format!("{:width$} {}", "Tags:", &buy.tags).green());
    println!(
        "{}",
        format!("{:width$} {:.2} (UAH)", "Price:", &buy.price).green()
    );
}

pub fn category_info(info: CategoryInfo) {
    let width = 15;
    println!("{}", format!("\n~~ {} ~~", &info.name).green());
    println!("{}", format!("{:width$} {}", "ID:", &info.id).green());
    println!("{}", format!("{:width$} {}", "Code:", &info.abbr).green());
    println!(
        "{}",
        format!("{:width$} {}", "Bike count:", &info.bike_count).green()
    );
    println!(
        "{}",
        format!("{:width$} {:.2} (UAH)", "Total spend:", &info.total_spend).green()
    );
    println!(
        "{}",
        format!("{:width$} {}", "Ride count:", &info.ride_count).green()
    );
    println!(
        "{}",
        format!(
            "{:width$} {:.2} (km)",
            "Total distance:", &info.total_distance
        )
        .green()
    );
}

pub fn lub_info(lub: ChainLubricationList, bike_name: String) {
    let width = 11;
    println!("{}", "\n~~ Chain lubrication ~~".green());
    println!("{}", format!("{:width$} {}", "ID:", &lub.lub_id).green());
    println!("{}", format!("{:width$} {}", "Bike:", &bike_name).green());
    println!("{}", format!("{:width$} {}", "Code:", &lub.bike).green());
    println!("{}", format!("{:width$} {}", "Date:", &lub.date).green());
    println!(
        "{}",
        format!("{:width$} {:.2} (km)", "Passed:", &lub.passed).green()
    );
    println!(
        "{}",
        format!("{:width$} {}", "Annotation:", &lub.annotation).green()
    );
}

pub fn ride_info(report: RidesInfoReport) {
    println!("{}", "\n~~ Rides ~~".green());

    let width: usize = cmp::max(
        report
            .distance_by_categories
            .keys()
            .max_by_key(|k| k.len())
            .unwrap_or(&"".to_string())
            .len()
            + 2,
        15,
    );

    if let Some(date) = report.date_eq {
        println!("{}", format!("at:  {}", &date).green());
    } else {
        let mut date_str: String = String::new();

        if let Some(date) = report.date_gt {
            date_str += &format!("from: {}", &date)
        }

        if let Some(date) = report.date_lt {
            date_str += &format!("  to: {}", &date)
        }

        if !date_str.is_empty() {
            println!("{}", date_str.green());
        }
    }

    println!(
        "{}",
        format!("{:width$} {}", "Rides count:", &report.rides_count).green()
    );
    println!(
        "{}",
        format!("{:width$} {:.2} (km)", "Last ride:", &report.last_distance).green()
    );
    println!(
        "{}",
        format!("{:width$} {}", "on:", &report.last_date.unwrap()).green()
    );
    println!(
        "{}",
        format!(
            "{:width$} {:.2} (km)",
            "Total distance:", &report.total_distance
        )
        .green()
    );

    if !report.iter_type.is_empty() {
        match report.iter_type.as_str() {
            "cat" => {
                println!("{}", "\nBy category:".green());
            }
            "bike" => {
                println!("{}", "\nBy bike:".green());
            }
            _ => {}
        }

        for (cat, distance) in report.distance_by_categories.iter() {
            println!(
                "{}",
                format!("{:width$} {:.2} (km)", &cat, &distance).green()
            );
        }
    }
}

pub fn ride_info_single(ride: &RideInfo) {
    let width = 11;
    println!("{}", "\n~~ Ride ~~".green());
    println!("{}", format!("{:width$} {}", "Count:", 1).green());
    println!("{}", format!("{:width$} {}", "ID:", &ride.ride_id).green());
    println!("{}", format!("{:width$} {}", "Bike:", &ride.bike).green());
    println!("{}", format!("{:width$} {}", "Date:", &ride.date).green());
    println!(
        "{}",
        format!("{:width$} {:.2} (km)", "Distance:", &ride.distance).green()
    );
    println!("{}", format!("{:width$} {}", "Tags:", &ride.tags).green());
    println!(
        "{}",
        format!("{:width$} {}", "Annotation:", &ride.annotation).green()
    );
}
