use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use sqlite::{Connection, State};

use crate::events::{Category, Event};
use crate::providers::EventProvider;
use crate::filters::EventFilter;

pub struct SQLiteProvider {
    name: String,
    path: PathBuf,
}

impl SQLiteProvider {
    pub fn new(name: &str, path: &Path) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_path_buf(),
        }
    }

    fn get_categories(&self, connection: &Connection) -> HashMap<i64, Category> {
        let mut category_map: HashMap<i64, Category> = HashMap::new();

        let query = "SELECT category_id, primary_name, secondary_name FROM category";
        let mut statement = connection.prepare(query).unwrap();

        while let Ok(State::Row) = statement.next() {
            let category_id = statement.read::<i64, _>("category_id").unwrap();
            let primary = statement.read::<String, _>("primary_name").unwrap();
            let secondary = statement.read::<Option<String>, _>("secondary_name").unwrap();

            let category = match secondary {
                Some(sec) => Category::new(&primary, &sec),
                None => Category::from_primary(&primary),
            };

            category_map.insert(category_id, category);
        }

        category_map
    }
}

impl EventProvider for SQLiteProvider {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn get_events(&self, filter: &EventFilter, events: &mut Vec<Event>) {  // muutos
    let connection = Connection::open(self.path.clone()).unwrap();
    let category_map = self.get_categories(&connection);

    let query = "SELECT event_date, event_description, category_id FROM event".to_string();
    let mut statement = connection.prepare(query).unwrap();

    while let Ok(State::Row) = statement.next() {
        let date_string = statement.read::<String, _>("event_date").unwrap();
        let date = NaiveDate::parse_from_str(&date_string, "%F").unwrap();
        let description = statement.read::<String, _>("event_description").unwrap();
        let category_id = statement.read::<i64, _>("category_id").unwrap();
        let category = category_map.get(&category_id).unwrap();

        let event = Event::new_singular(date, description.to_string(), category.clone());
        if filter.accepts(&event) {  // suodatus tässä
            events.push(event);
        }
    }
}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::Category;

    fn create_memory_db() -> sqlite::Connection {
        let connection = sqlite::open(":memory:").unwrap();
        let query = "
CREATE TABLE IF NOT EXISTS event(
    event_id INTEGER PRIMARY KEY,
    event_date DATE NOT NULL,
    event_description TEXT NOT NULL,
    category_id INTEGER NOT NULL,
    FOREIGN KEY (category_id) REFERENCES category(category_id));
CREATE TABLE IF NOT EXISTS category(
    category_id INTEGER PRIMARY KEY,
    primary_name TEXT NOT NULL,
    secondary_name TEXT);
INSERT INTO category VALUES (1, 'test', NULL);
INSERT INTO event (event_date, event_description, category_id)
    VALUES ('2026-03-07', 'Unit test for SQLiteProvider', 1);
";
        connection.execute(query).unwrap();
        connection
    }

    #[test]
    fn test_get_categories() -> Result<(), String> {
        let connection = create_memory_db();
        let query = "SELECT category_id, primary_name, secondary_name FROM category";
        let mut statement = connection.prepare(query).unwrap();

        if let Ok(sqlite::State::Row) = statement.next() {
            let actual = (
                statement.read::<i64, _>("category_id").unwrap(),
                statement.read::<String, _>("primary_name").unwrap(),
                statement.read::<Option<String>, _>("secondary_name").unwrap(),
            );
            let expected = (1, "test".to_string(), None);
            assert_eq!(expected, actual);
            Ok(())
        } else {
            Err("Unable to get category from database".to_string())
        }
    }

    #[test]
    fn get_events() -> Result<(), String> {
        let connection = create_memory_db();
        let query =
            "SELECT event_date, event_description, category_id FROM event".to_string();
        let mut statement = connection.prepare(query).unwrap();

        if let Ok(sqlite::State::Row) = statement.next() {
            let expected = ("2026-03-07".to_string(), "Unit test for SQLiteProvider".to_string(), 1);
            let actual = (
                statement.read::<String, _>("event_date").unwrap(),
                statement.read::<String, _>("event_description").unwrap(),
                statement.read::<i64, _>("category_id").unwrap(),
            );
            assert_eq!(expected, actual);
            Ok(())
        } else {
            Err("Unable to retrieve event from database".to_string())
        }
    }
}
