#![allow(dead_code)]
#![allow(unused)]
pub mod myprovider;
pub mod sqlite;
pub mod textfile;
pub mod csvfile;
pub mod web;

use::std::fmt;
use crate::events::Event;
use crate::filters::EventFilter;

// Virheet, joita lisäys-operaatio (add_event) voi palauttaa
#[derive(Debug)]
pub enum EventProviderError {
    // Provider ei ylipäänsä tue lisäystä (esim. WebProvider)
    AddNotSupported,
    // I/O-virhe tiedostoa kirjoitettaessa
    IoError(String),
}

impl fmt::Display for EventProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventProviderError::AddNotSupported => {
                write!(f, "this provider does not support adding events")
            }
            EventProviderError::IoError(msg) => {
                write!(f, "I/O error: {}", msg)
            }
        }
    }
}

impl std::error::Error for EventProviderError {}

pub trait EventProvider {
    fn name(&self) -> String;
    fn get_events(&self, filter: &EventFilter, events: &mut Vec<Event>);

    // Kertooko provider tukevansa tapahtumien lisäystä
    // Oletuksena false, lisäystä tukevat providerit voivat
    // ylikirjoittaa tämän palauttamaan true
    fn is_add_supported(&self) -> bool {
        false
    }
    fn add_event(&self, _event: &Event) -> Result<(), EventProviderError> {
        Err(EventProviderError::AddNotSupported)
    }
}