use owo_colors::OwoColorize;

use crate::{
    db::models::{BikeInfo, CategoryInfo, ChainLubricationList},
    handlers::helpers::get::bike,
};

pub fn ride_info(bike: BikeInfo) {
    let after_lub_distance: f32 = bike.after_lub_distance;
    let msg: String = format!(
        "Without chain lubrication, passed: {} km",
        &after_lub_distance
    );

    println!("{}", format!("\n~~ {} ~~", &bike.name).green());
    println!(
        "{}",
        format!("Category:         {}", &bike.category).green()
    );
    println!("{}", format!("ID:               {}", &bike.id).green());
    println!("{}", format!("Bike code:        {}", &bike.code).green());
    println!(
        "{}",
        format!("Added:            {}", &bike.add_date).green()
    );
    println!(
        "{}",
        format!("Total spend:      {} UAH", &bike.total_spend).green()
    );
    println!(
        "{}",
        format!("Ride count:       {}", &bike.ride_count).green()
    );
    if let Some(date) = bike.last_ride {
        println!(
            "{}",
            format!("Total distance:   {} km", &bike.total_distance).green()
        );
        println!("{}", format!("Last ride:        {}", &date).green());
        println!(
            "{}",
            format!("    distance:     {} km", &bike.last_distance).green()
        );
    }
    if let Some(date) = bike.maintenance {
        println!("{}", format!("Last maintenance: {}", &date).green());
    }
    if let Some(date) = bike.chain_lub {
        println!("{}", format!("Last chain lub:   {}", &date).green());
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

pub fn category_info(info: CategoryInfo) {
    println!("{}", format!("\n~~ {} ~~", &info.name).green());
    println!("{}", format!("ID:             {}", &info.id).green());
    println!("{}", format!("Code:           {}", &info.abbr).green());
    println!(
        "{}",
        format!("Bike count:     {}", &info.bike_count).green()
    );
    println!(
        "{}",
        format!("Total spend:    {} UAH", &info.total_spend).green()
    );
    println!(
        "{}",
        format!("Ride count:     {}", &info.ride_count).green()
    );
    println!(
        "{}",
        format!("Total distance: {} km", &info.total_distance).green()
    );
}

pub fn lub_info(lub: ChainLubricationList, bike_name: String) {
    println!("{}", "\n~~ Chain lubrication ~~".green());
    println!("{}", format!("ID:         {}", &lub.lub_id).green());
    println!("{}", format!("Bike:       {}", &bike_name).green());
    println!("{}", format!("Code:       {}", &lub.bike).green());
    println!("{}", format!("Date:       {}", &lub.date).green());
    println!("{}", format!("Passed:     {}", &lub.passed).green());
    println!("{}", format!("Annotation: {}", &lub.annotation).green());
}
