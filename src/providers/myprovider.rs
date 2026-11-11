#![allow(dead_code)]
use crate::events::{self, Category, Event};
use crate::providers::EventProvider;
use crate::filters::EventFilter;
use chrono::NaiveDate;

pub struct MyProvider {
    name: String,
}

impl MyProvider {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }
}

impl EventProvider for MyProvider {
    fn name(&self) -> String {
        self.name.clone()
    }

   fn get_events(&self, filter: &EventFilter, events: &mut Vec<Event>) {  // muutos
        let all_events = vec![
            Event::new_singular(
                NaiveDate::from_ymd_opt(2025, 12, 11).unwrap(),
                String::from("Death of Wes Craven"),
                Category::from_str("movies/horror")),
            Event::new_singular(
                NaiveDate::from_ymd_opt(2015, 5, 15).unwrap(),
                String::from("Scream 4 released"),
                Category::new("movies", "horror")),
            Event::new_singular(
                NaiveDate::from_ymd_opt(2026, 3, 12).unwrap(),
                String::from("Pentagon tells Congress the Iran war cost at least $11B in first six days"),
                Category::from_str("war/USA")),
        ];

        for event in all_events {
            if filter.accepts(&event) {  // suodatus tässä
                events.push(event);
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::MyProvider;
    use crate::providers::EventProvider;
    use crate::filters::FilterBuilder;
    use crate::events::Event;

    #[test]
    fn get_events_returns_three_events() {
        let provider = MyProvider::new("test");
        let mut events: Vec<Event> = Vec::new();
        let filter = FilterBuilder::new().build();
        provider.get_events(&filter, &mut events);
        assert_eq!(events.len(), 3);
    }
}