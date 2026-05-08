use indexmap::IndexMap;

use crate::handlers::structs::{BuysInfoReport, RidesInfoReport};

struct GraphData {
    title: String,
    data: IndexMap<String, f32>,
    count: u32,
    total_val: f32,
    units: String,
    count_title: String,
    val_title: String,
    uncategorized: Option<f32>,
}

pub fn ride_graph(report: RidesInfoReport) {
    let title_text: String = format!("{:>9} {} ~~", "~~ Rides", &report.iter_type);
    let val_title_text: String = format!("{:15}", "total distance:");
    let count_title_text: String = format!("{:15}", "ride count:");

    let graph_data: GraphData = GraphData {
        title: title_text,
        data: report.distance_by_categories,
        count: report.rides_count,
        total_val: report.total_distance,
        units: String::from("km"),
        count_title: count_title_text,
        val_title: val_title_text,
        uncategorized: None,
    };

    graph_h(graph_data);
}

pub fn buy_graph(report: BuysInfoReport) {
    let title_text: String = format!("{:>9} {} ~~", "~~ Buys", &report.iter_type);
    let val_title_text: String = format!("{:15}", "total spend:");
    let count_title_text: String = format!("{:15}", "buys count:");
    let uncat: Option<f32> = if report.spend_uncategorized > 0.0 {
        Some(report.spend_uncategorized)
    } else {
        None
    };

    let graph_data: GraphData = GraphData {
        title: title_text,
        data: report.spend_by_categories,
        count: report.buys_count,
        total_val: report.total_spend,
        units: String::from("UAH"),
        count_title: count_title_text,
        val_title: val_title_text,
        uncategorized: uncat,
    };

    graph_h(graph_data);
}

fn graph_h(data: GraphData) {
    let mut max_val = data
        .data
        .iter()
        .map(|(_, v)| *v)
        .reduce(f32::max)
        .unwrap_or(0.);

    if let Some(uncat) = data.uncategorized {
        max_val = max_val.max(uncat);
    }

    let max_len = 100.0;
    let len_point = max_val / max_len;

    println!("{}", &data.title);
    println!("{:>9}", "│");

    fn get_formatted_line(key: String, val: f32, len_point: f32) -> String {
        let len: usize = (val / len_point).ceil() as usize;
        let mut bar_len = len / 2;
        let mut bar: String = "▄".repeat(bar_len);
        if !len.is_multiple_of(2) {
            bar_len += 1;
            bar += "▖";
        }
        format!("{:>8}│{}{} {}", &key, bar, " ".repeat(52 - (bar_len)), val)
    }

    for (key, val) in data.data {
        println!("{}", get_formatted_line(key, val, len_point));
    }

    if let Some(val) = data.uncategorized {
        println!(
            "{}",
            get_formatted_line(String::from("Uncat:"), val, len_point)
        );
    }

    println!("{:>9}{}({})", "└", "─".repeat(60), &data.units);
    println!("{:>9}{} {}", "", &data.count_title, &data.count);
    println!(
        "{:>9}{} {}{}",
        "", &data.val_title, &data.total_val, &data.units
    );
}
