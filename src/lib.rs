//raskaasti kirjan inspiroima
pub mod events;
pub mod filters;
mod birthday;
mod providers;

use std::error::Error;
use std::path::Path;
use chrono::NaiveDate;
use serde::Deserialize;
use crate::events::{Event, Category, MonthDay};
use crate::providers::{EventProvider, myprovider::MyProvider};
use crate::providers::sqlite::SQLiteProvider;
use crate::providers::textfile::TextFileProvider;
use crate::providers::csvfile::CSVFileProvider;
use crate::providers::web::WebProvider;

#[derive(Deserialize, Debug)]
pub struct ProviderConfig {
    pub name: String,
    pub kind: String,
    pub resource: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub providers: Vec<ProviderConfig>,
}

// Apufunktio: rakentaa provider-instanssin sen tyypin ja konfiguraation perusteella
fn create_provider(
    provider_config: &ProviderConfig,
    config_path: &Path,
) -> Option<Box<dyn EventProvider>> {
    match provider_config.kind.as_str() {
        "MyProvider" => Some(Box::new(MyProvider::new(&provider_config.name))),
        "sqlite" => {
            let path = config_path.join(&provider_config.resource);
            Some(Box::new(SQLiteProvider::new(&provider_config.name, &path)))
        }
        "text" => {
            let path = config_path.join(&provider_config.resource);
            Some(Box::new(TextFileProvider::new(&provider_config.name, &path)))
        }
        "csv" => {
            let path = config_path.join(&provider_config.resource);
            Some(Box::new(CSVFileProvider::new(&provider_config.name, &path)))
        }
        "web" => {
            Some(Box::new(WebProvider::new(&provider_config.name, &provider_config.resource)))
        }
        unknown => {
            eprintln!("Unknown provider kind: {}", unknown);
            None
        }
    }
}

pub fn run(
    config: &Config,
    date_filter: &Option<MonthDay>,
    config_path: &Path,
    no_birthday: bool,
    exclude_list: &[Category],
) -> Result<(), Box<dyn Error>> {
    use chrono::{Local, Datelike};
    use crate::filters::FilterBuilder;

    // Syntymäpäivälaskenta tehdään ENNEN tapahtumien näyttämistä,
    // ellei käyttäjä ole pyytänyt sen ohittamista --no-birthday-flagilla
    if !no_birthday {
        birthday::handle_birthday();
    }

    let month_day = match date_filter {
        Some(md) => md.clone(),
        None => {
            let today = Local::now().date_naive();
            MonthDay::new(today.month(), today.day())
        }
    };

    // Rakennetaan filtteri. Kuukausi+päivä on aina, ekskluusiot vain jos annettu
    let mut builder = FilterBuilder::new().month_day(month_day);
    for category in exclude_list {
        builder = builder.exclude_category(category.clone());
    }
    let filter = builder.build();

    let mut events: Vec<Event> = Vec::new();

    for provider_config in &config.providers {
        if let Some(provider) = create_provider(provider_config, config_path) {
            provider.get_events(&filter, &mut events);
        }
    }

    for event in &events {
        println!("{}", event);
    }

    Ok(())
}

// Listaa providerit ja kertoo kunkin tukevuuden lisäystoiminnolle
pub fn list_providers(config: &Config, config_path: &Path) {
    for provider_config in &config.providers {
        let support = match create_provider(provider_config, config_path) {
            Some(provider) => {
                if provider.is_add_supported() {
                    "supports add"
                } else {
                    "read-only"
                }
            }
            None => "unknown",
        };
        println!(
            "- {} ({}) [{}]",
            provider_config.name, provider_config.kind, support
        );
    }
}

// Lisää tapahtuman nimettyyn provideriin
pub fn add_event(
    config: &Config,
    config_path: &Path,
    provider_name: &str,
    date: NaiveDate,
    description: String,
    category: Category,
) -> Result<(), Box<dyn Error>> {
    // Etsitään konfiguraatiosta provider annetulla nimellä
    let provider_config = config
        .providers
        .iter()
        .find(|p| p.name == provider_name)
        .ok_or_else(|| format!("Provider '{}' not found in configuration", provider_name))?;

    let provider = create_provider(provider_config, config_path)
        .ok_or_else(|| format!("Could not create provider '{}'", provider_name))?;

    if !provider.is_add_supported() {
        return Err(format!(
            "Provider '{}' ({}) does not support adding events",
            provider_name, provider_config.kind
        )
        .into());
    }

    let event = Event::new_singular(date, description, category);
    provider.add_event(&event)?;

    println!("Event added to '{}'", provider_name);
    Ok(())
}