use chrono::NaiveDate;

use crate::cli::structs::Date;

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
