use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use csv::{ReaderBuilder, WriterBuilder};

use crate::events::{Category, Event};
use crate::filters::EventFilter;
use crate::providers::{EventProvider, EventProviderError};

pub struct CSVFileProvider {
    name: String,
    path: PathBuf,
}

impl CSVFileProvider {
    pub fn new(name: &str, path: &Path) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_path_buf(),
        }
    }
}

impl EventProvider for CSVFileProvider {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn get_events(&self, filter: &EventFilter, events: &mut Vec<Event>) {
        let mut reader = match ReaderBuilder::new()
            .has_headers(true)
            .from_path(&self.path)
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Could not open CSV file '{}': {}", self.path.display(), e);
                return;
            }
        };

        for result in reader.records() {
            let record = match result {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Error reading CSV row: {}", e);
                    continue;
                }
            };

            // Odotamme tarkalleen kolme kenttää: date, description, category
            if record.len() != 3 {
                eprintln!(
                    "Skipping malformed CSV row in {}: expected 3 fields, got {}",
                    self.path.display(),
                    record.len()
                );
                continue;
            }

            let date_string = &record[0];
            let description = &record[1];
            let category_string = &record[2];

            match NaiveDate::parse_from_str(date_string, "%Y-%m-%d") {
                Ok(date) => {
                    let category = Category::from_str(category_string);
                    let event = Event::new_singular(
                        date,
                        description.to_string(),
                        category,
                    );
                    if filter.accepts(&event) {
                        events.push(event);
                    }
                }
                Err(_) => {
                    eprintln!(
                        "Invalid date '{}' in {}",
                        date_string,
                        self.path.display()
                    );
                }
            }
        }
    }

    fn is_add_supported(&self) -> bool {
        true
    }

    fn add_event(&self, event: &Event) -> Result<(), EventProviderError> {
        // Tarkistetaan onko tiedosto jo olemassa, jotta tiedämme tarvitseeko
        // kirjoittaa otsikkorivi vai ei.
        let file_exists = self.path.exists();

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| EventProviderError::IoError(format!(
                "could not open '{}' for appending: {}",
                self.path.display(),
                e
            )))?;

        // has_headers(false) koska olemme jo päättäneet manuaalisesti
        // kirjoitetaanko otsikkorivi vai ei.
        let mut writer = WriterBuilder::new()
            .has_headers(false)
            .from_writer(file);

        // Jos tiedostoa ei ollut olemassa, kirjoitetaan otsikkorivi ensin
        if !file_exists {
            writer
                .write_record(&["date", "description", "category"])
                .map_err(|e| EventProviderError::IoError(format!(
                    "could not write CSV header to '{}': {}",
                    self.path.display(),
                    e
                )))?;
        }

        // Kirjoitetaan tapahtuma yhdelle riville
        let date_string = event.date().to_string();
        let description = event.description();
        let category_string = event.category().to_string();

        writer
            .write_record(&[date_string, description, category_string])
            .map_err(|e| EventProviderError::IoError(format!(
                "could not write CSV record to '{}': {}",
                self.path.display(),
                e
            )))?;

        // Pakottaa puskurin sisällön levylle
        writer.flush().map_err(|e| EventProviderError::IoError(format!(
            "could not flush '{}': {}",
            self.path.display(),
            e
        )))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    fn create_temp_file(name: &str, content: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(name);
        let mut file = File::create(&path).expect("create temp file");
        file.write_all(content.as_bytes()).expect("write content");
        path
    }

    // Apufunktio joka palauttaa polun tyhjään väliaikaistiedostoon.
    // Tarvitaan testissä, jossa tiedosto pitää olla olematta olemassa.
    fn temp_file_path(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(name);
        // Jos jokin aiempi testikierros jätti tiedoston, poistetaan se
        let _ = std::fs::remove_file(&path);
        path
    }

    #[test]
    fn read_one_event() {
        let content = "\
date,description,category
2025-03-23,Elfyn Evans wins WRC Safari Rally Kenya,rally/wrc
";
        let path = create_temp_file("today_test_csv_one.csv", content);

        let provider = CSVFileProvider::new("test", &path);
        let mut events: Vec<Event> = Vec::new();
        let filter = crate::filters::FilterBuilder::new().build();

        provider.get_events(&filter, &mut events);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].description(), "Elfyn Evans wins WRC Safari Rally Kenya");
    }

    #[test]
    fn read_multiple_events() {
        let content = "\
date,description,category
1989-07-17,B-2 Spirit takes its first flight,military/aviation
1972-05-10,A-10 Thunderbolt II takes its first flight,military/aviation
2022-10-02,Kalle Rovanperä becomes WRC champion,rally/wrc
";
        let path = create_temp_file("today_test_csv_multi.csv", content);

        let provider = CSVFileProvider::new("test", &path);
        let mut events: Vec<Event> = Vec::new();
        let filter = crate::filters::FilterBuilder::new().build();

        provider.get_events(&filter, &mut events);

        assert_eq!(events.len(), 3);
    }

    #[test]
    fn invalid_date_is_skipped() {
        let content = "\
date,description,category
not-a-date,This event has bad date,test/category
1972-05-10,A-10 Thunderbolt II takes its first flight,military/aviation
";
        let path = create_temp_file("today_test_csv_baddate.csv", content);

        let provider = CSVFileProvider::new("test", &path);
        let mut events: Vec<Event> = Vec::new();
        let filter = crate::filters::FilterBuilder::new().build();

        provider.get_events(&filter, &mut events);

        assert_eq!(events.len(), 1);
    }

    #[test]
    fn add_event_creates_file_with_header() {
        // Käytetään tiedostoa jota ei ole vielä olemassa
        let path = temp_file_path("today_test_csv_add_new.csv");

        let provider = CSVFileProvider::new("test", &path);
        assert!(provider.is_add_supported());

        let date = NaiveDate::from_ymd_opt(2025, 3, 23).unwrap();
        let category = Category::from_str("rally/wrc");
        let event = Event::new_singular(
            date,
            "Elfyn Evans wins WRC Safari Rally Kenya".to_string(),
            category,
        );
        let result = provider.add_event(&event);
        assert!(result.is_ok());

        // Luetaan se takaisin
        let mut events: Vec<Event> = Vec::new();
        let filter = crate::filters::FilterBuilder::new().build();
        provider.get_events(&filter, &mut events);

        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].description(),
            "Elfyn Evans wins WRC Safari Rally Kenya"
        );
    }

    #[test]
    fn add_event_appends_to_existing_file() {
        // Tiedostossa on jo kaksi tapahtumaa
        let initial_content = "\
date,description,category
1989-07-17,B-2 Spirit takes its first flight,military/aviation
1972-05-10,A-10 Thunderbolt II takes its first flight,military/aviation
";
        let path = create_temp_file("today_test_csv_add_existing.csv", initial_content);

        let provider = CSVFileProvider::new("test", &path);

        let date = NaiveDate::from_ymd_opt(2025, 3, 23).unwrap();
        let category = Category::from_str("rally/wrc");
        let event = Event::new_singular(
            date,
            "Elfyn Evans wins Safari Rally".to_string(),
            category,
        );
        let result = provider.add_event(&event);
        assert!(result.is_ok());

        // Pitäisi nyt olla 3 tapahtumaa
        let mut events: Vec<Event> = Vec::new();
        let filter = crate::filters::FilterBuilder::new().build();
        provider.get_events(&filter, &mut events);

        assert_eq!(events.len(), 3);
    }
}