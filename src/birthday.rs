use chrono::{Datelike, Local, NaiveDate};
use std::env;

pub fn make_message(days_diff: i64) -> String {
    if days_diff > 0 {
        if days_diff % 1000==0{
            format!("You are {} days old. That's a nice, round number!", days_diff)
        } else {
            format!("You are {} days old.", days_diff)
        }
    } else if days_diff < 0 {
        "Are you from the future?".to_string()
    } else {
        "Looks like you're new here.".to_string()
    }
}
// Lukee BIRTHDATE-ympäristömuuttujan, vertaa sitä tämänhetkiseen päivämäärään
pub fn handle_birthday() {
    let birthdate_str = match env::var("BIRTHDATE") {
        Ok(val) => val,
        Err(_) => {
            println!("BIRTHDATE environment variable not set.");
            return;
        }
    };

    let birthdate = match NaiveDate::parse_from_str(&birthdate_str, "%Y-%m-%d") {
        Ok(date) => date,
        Err(_) => {
            println!("Invalid date format in BIRTHDATE. Expected YYYY-MM-DD.");
            return;
        }
    };

    let today = Local::now().date_naive();

    // Onko syntymäpäivä tänään? Tulostetaan onnittelu samalle riville
    if birthdate.month() == today.month() && birthdate.day() == today.day() {
        print!("Happy birthday! ");
    }

    let days_diff = today.signed_duration_since(birthdate).num_days();
    println!("{}", make_message(days_diff));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_message_normal() {
        assert_eq!(make_message(12345), "You are 12345 days old.");
    }

    #[test]
    fn make_message_round_number() {
        assert_eq!(
            make_message(10000),
            "You are 10000 days old. That's a nice, round number!"
        );
    }

    #[test]
    fn make_message_newborn() {
        assert_eq!(make_message(0), "Looks like you're new here.");
    }

    #[test]
    fn make_message_future() {
        assert_eq!(make_message(-1), "Are you from the future?");
    }

    #[test]
    fn make_message_one_day_old() {
        // varmistaa ettei 1 mene round-numberiksi
        assert_eq!(make_message(1), "You are 1 days old.");
    }

    #[test]
    fn make_message_thousand() {
        assert_eq!(
            make_message(1000),
            "You are 1000 days old. That's a nice, round number!"
        );
    }
}