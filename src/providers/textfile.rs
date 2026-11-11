use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use chrono::NaiveDate;

use crate::events::{Category, Event};
use crate::filters::EventFilter;
use crate::providers::{EventProvider, EventProviderError};

enum ReadingState {
    Date,
    Description,
    Category,
    Separator,
}

pub struct TextFileProvider {
    name: String,
    path: PathBuf,
}

impl TextFileProvider {
    pub fn new(name: &str, path: &Path) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_path_buf(),
        }
    }
}

impl EventProvider for TextFileProvider {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn get_events(&self, filter: &EventFilter, events: &mut Vec<Event>) {
        let file = match File::open(&self.path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Could not open text file '{}': {}", self.path.display(), e);
                return;
            }
        };

        let reader = BufReader::new(file);

        let mut state = ReadingState::Date;
        let mut date_string = String::new();
        let mut description = String::new();
        let mut category_string = String::new();

        for line_result in reader.lines() {
            let line = match line_result {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Error reading line: {}", e);
                    continue;
                }
            };

            match state {
                ReadingState::Date => {
                    date_string = line;
                    state = ReadingState::Description;
                }
                ReadingState::Description => {
                    description = line;
                    state = ReadingState::Category;
                }
                ReadingState::Category => {
                    category_string = line;
                    state = ReadingState::Separator;
                }
                ReadingState::Separator => {
                    // Erotinrivi: aiemmin kerätyistä tiedoista rakennetaan tapahtuma
                    match NaiveDate::parse_from_str(&date_string, "%Y-%m-%d") {
                        Ok(date) => {
                            let category = Category::from_str(&category_string);
                            let event = Event::new_singular(
                                date,
                                std::mem::take(&mut description),
                                category,
                            );
                            if filter.accepts(&event) {
                                events.push(event);
                            }
                        }
                        Err(_) => {
                            eprintln!("Invalid date '{}' in {}", date_string, self.path.display());
                        }
                    }
                    state = ReadingState::Date;
                }
            }
        }
    }

    fn is_add_supported(&self) -> bool {
        true
    }

    fn add_event(&self, event: &Event) -> Result<(), EventProviderError> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| EventProviderError::IoError(format!(
                "could not open '{}' for appending: {}",
                self.path.display(),
                e
            )))?;

        // Kirjoitetaan tapahtuma neljän rivin lohkona
        // päivämäärä, kuvaus, kategoria, tyhjä rivi
        let block = format!(
            "{}\n{}\n{}\n\n",
            event.date(),
            event.description(),
            event.category()
        );

        file.write_all(block.as_bytes())
            .map_err(|e| EventProviderError::IoError(format!(
                "could not write to '{}': {}",
                self.path.display(),
                e
            )))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Apufunktio joka luo väliaikaisen tekstitiedoston annetulla sisällöllä
    /// ja palauttaa polun siihen
    fn create_temp_file(name: &str, content: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(name);
        let mut file = File::create(&path).expect("create temp file");
        file.write_all(content.as_bytes()).expect("write content");
        path
    }

    #[test]
    fn read_one_event() {
        let content = "2025-03-23\nElfyn Evans wins WRC Safari Rally Kenya\nrally/wrc\n\n";
        let path = create_temp_file("today_test_one.txt", content);

        let provider = TextFileProvider::new("test", &path);
        let mut events: Vec<Event> = Vec::new();
        let filter = crate::filters::FilterBuilder::new().build();

        provider.get_events(&filter, &mut events);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].description(), "Elfyn Evans wins WRC Safari Rally Kenya");
    }

    #[test]
    fn read_multiple_events() {
        let content = "\
1989-07-17
B-2 Spirit stealth bomber takes its first flight
military/aviation

1972-05-10
A-10 Thunderbolt II takes its first flight
military/aviation

";
        let path = create_temp_file("today_test_multi.txt", content);

        let provider = TextFileProvider::new("test", &path);
        let mut events: Vec<Event> = Vec::new();
        let filter = crate::filters::FilterBuilder::new().build();

        provider.get_events(&filter, &mut events);

        assert_eq!(events.len(), 2);
    }

    #[test]
    fn invalid_date_is_skipped() {
        let content = "\
not-a-date
This event has bad date
test/category

1972-05-10
A-10 Thunderbolt II takes its first flight
military/aviation

";
        let path = create_temp_file("today_test_baddate.txt", content);

        let provider = TextFileProvider::new("test", &path);
        let mut events: Vec<Event> = Vec::new();
        let filter = crate::filters::FilterBuilder::new().build();

        provider.get_events(&filter, &mut events);

        // Vain toinen tapahtuma kelpaa, ensimmäinen hylätään virheellisen päiväyksen takia
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn add_event_appends_to_file() {
        // Aloitetaan tyhjästä tiedostosta
        let path = create_temp_file("today_test_add.txt", "");

        let provider = TextFileProvider::new("test", &path);
        assert!(provider.is_add_supported());

        // Lisätään tapahtuma
        let date = NaiveDate::from_ymd_opt(2025, 3, 23).unwrap();
        let category = Category::from_str("rally/wrc");
        let event = Event::new_singular(
            date,
            "Elfyn Evans wins WRC Safari Rally Kenya".to_string(),
            category,
        );
        let result = provider.add_event(&event);
        assert!(result.is_ok());

        // Luetaan se takaisin samalla providerilla
        let mut events: Vec<Event> = Vec::new();
        let filter = crate::filters::FilterBuilder::new().build();
        provider.get_events(&filter, &mut events);

        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].description(),
            "Elfyn Evans wins WRC Safari Rally Kenya"
        );
    }
}