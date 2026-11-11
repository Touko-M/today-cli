#![allow(dead_code)]
use std::fmt;
use chrono::{NaiveDate, Datelike};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MonthDay {
    pub month: u32,
    pub day: u32,
}

impl MonthDay {
    pub fn new(month: u32, day: u32) -> Self {
        Self { month, day }
    }

    pub fn from_str(s: &str) -> Self {
        assert!(s.len() == 4);
        let month_string = &s[..2];
        let month = month_string.parse().unwrap();
        let day: u32 = s[2..].parse().unwrap();
        MonthDay { month, day }
    }
    pub fn parse_from_str(s: &str) -> Result<MonthDay, String>{
        //poistetaan väliviiva jos on
        let cleaned: String = s.chars().filter(|c| *c != '-').collect();
        if cleaned.len() != 4 {
            return Err(format!("Invalid date format: '{}'. Expected MM--DD or MMDD", s));
        }
        let month: u32 = cleaned[..2].parse().map_err(|_| format!("Invalid month in '{}'",s))?;
        let day: u32 = cleaned[2..].parse().map_err(|_| format!("Invalid day in '{}",s))?;

        if month < 1 || month > 12 {
            return Err(format!("Month must be 1-12, got {}", month));
        }
        if day < 1 || day > 31 {
            return Err(format!("Day must be 1-31. got '{}'", day));
        }
        Ok(MonthDay{ month, day})
     }
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Category {
    primary: String,
    secondary: Option<String>,
}

impl Category {
    pub fn new(primary: &str, secondary: &str) -> Self {
        Self {
            primary: primary.to_string(),
            secondary: Some(secondary.to_string()),
        }
    }

    pub fn from_primary(primary: &str) -> Self {
        Self {
            primary: primary.to_string(),
            secondary: None,
        }
    }

    pub fn from_str(s: &str) -> Category {
        let parts: Vec<&str> = s.split("/").collect();
        if parts.len() < 2 {
            Category { primary: parts[0].to_string(), secondary: None }
        } else {
            Category { primary: parts[0].to_string(), secondary: Some(parts[1].to_string()) }
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.secondary {
            Some(sec) => write!(f, "{}/{}", self.primary, sec),
            None => write!(f, "{}", self.primary),
        }
    }
}

#[derive(Debug)]
pub enum EventKind {
    Singular(NaiveDate),
}

#[derive(Debug)]
pub struct Event {
    kind: EventKind,
    description: String,
    category: Category,
}

impl Event {
    pub fn new_singular(date: NaiveDate, description: String, category: Category) -> Self {
        Event {
            kind: EventKind::Singular(date),
            description,
            category,
        }
    }

    pub fn year(&self) -> i32 {
        match &self.kind {
            EventKind::Singular(date) => date.year(),
        }
    }

    pub fn month_day(&self) -> MonthDay {
        match &self.kind {
            EventKind::Singular(date) =>
                MonthDay { month: date.month(), day: date.day() },
        }
    }
    pub fn category(&self) -> Category {
    self.category.clone()
    }
   pub fn description(&self) -> String {
        self.description.clone()
    }

    /// Palauttaa tapahtuman päivämäärän. Käytetään mm. add_event:ssä
    pub fn date(&self) -> NaiveDate {
        match &self.kind {
            EventKind::Singular(date) => *date,
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {} ({})",
            match &self.kind {
                EventKind::Singular(date) => date,
            },
            self.description, self.category)
    }
}
