use chrono::{Datelike, NaiveDate};
use serde::Deserialize;

use crate::events::{Category, Event};
use crate::filters::EventFilter;
use crate::providers::EventProvider;

// Vain data-kenttä on meille tärkeä
#[derive(Deserialize, Debug)]
struct MuffinResponse {
    data: MuffinData,
}

// data-kentän sisältö. API palauttaa myös Births ja Deaths,
// mutta keskitymme Events-listaan.
#[derive(Deserialize, Debug)]
struct MuffinData {
    #[serde(rename = "Events")]
    events: Vec<MuffinEvent>,
}

// Yksittäisen tapahtuman rakenne API:n vastauksessa.
// Vuosi tulee merkkijonona ("1989" tai "42 BC"), jonka takia
// käsittelemme sen myöhemmin parsittaessa.
#[derive(Deserialize, Debug)]
struct MuffinEvent {
    year: String,
    text: String,
}

pub struct WebProvider {
    name: String,
    base_url: String,
}

impl WebProvider {
    pub fn new(name: &str, base_url: &str) -> Self {
        Self {
            name: name.to_string(),
            base_url: base_url.to_string(),
        }
    }
}

impl EventProvider for WebProvider {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn get_events(&self, filter: &EventFilter, events: &mut Vec<Event>) {
        // Otetaan kuukausi ja päivä filtteristä. Jos filtterissä ei
        // ole kuukautta+päivää, käytetään tätä päivää oletuksena.
        let (month, day) = match filter.month_day() {
            Some(md) => (md.month, md.day),
            None => {
                let today = chrono::Local::now().date_naive();
                (today.month(), today.day())
            }
        };

        let url = format!("{}/{}/{}", self.base_url, month, day);

        let response = match reqwest::blocking::get(&url) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Could not connect to '{}': {}", url, e);
                return;
            }
        };

        let muffin: MuffinResponse = match response.json() {
            Ok(j) => j,
            Err(e) => {
                eprintln!("Could not parse JSON from '{}': {}", url, e);
                return;
            }
        };

        for muffin_event in muffin.data.events {
            // Vuosi on merkkijonona. Yritetään parsia se kokonaisluvuksi;
            // BC-vuodet ja muut epäkelvolliset hylätään hiljaa.
            let year: i32 = match muffin_event.year.parse() {
                Ok(y) => y,
                Err(_) => continue,
            };

            let date = match NaiveDate::from_ymd_opt(year, month, day) {
                Some(d) => d,
                None => continue,
            };

            let category = Category::from_str("web/history");
            let event = Event::new_singular(date, muffin_event.text, category);

            if filter.accepts(&event) {
                events.push(event);
            }
        }
    }
}