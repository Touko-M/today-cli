# Today

A command-line tool in Rust that shows what happened on this day in history. Events come from several sources at once: text files, CSV files, an SQLite database and a public web API.

Course project for Advanced Programming Techniques at Tampere University of Applied Sciences (TAMK), spring 2026.

## Features

- Five event providers behind one `EventProvider` trait, used as `Box<dyn EventProvider>`: text file, CSV file, SQLite, web (Muffinlabs History API) and a hard-coded test provider
- Providers are configured in a TOML file, and several providers of the same kind can be used together
- Show events for today or for a given date, and exclude categories
- Add new events to text and CSV providers from the command line
- Optional message with your age in days, and a greeting on your birthday
- 31 unit tests

## Setup

The program reads its configuration and data from the platform's configuration directory. On Linux:

```bash
mkdir -p ~/.config/today
cp config-example/* ~/.config/today/
```

`today.toml` lists the providers. File and database paths are relative to the configuration directory, and the web provider takes a full URL:

```toml
[[providers]]
name = "history"
kind = "text"
resource = "today.txt"

[[providers]]
name = "history-csv"
kind = "csv"
resource = "today.csv"

[[providers]]
name = "history-db"
kind = "sqlite"
resource = "events.db"

[[providers]]
name = "muffinlabs"
kind = "web"
resource = "https://history.muffinlabs.com/date"
```

## Usage

```
Usage: today [OPTIONS] [COMMAND]

Commands:
  providers  List all event providers
  add        Adds an event to an event provider

Options:
  -d, --date <DATE>        Event date in MMDD format
  -e, --exclude <EXCLUDE>  Categories to exclude, comma-separated (a/b,c/d)
  -n, --no-birthday        No age calculation or birthday message
  -h, --help               Print help
```

Examples:

```bash
# Events for today, with the age message
BIRTHDATE=1990-01-31 cargo run -q

# Events for 10 May, without web/history events and without the age message
cargo run -q -- -d 0510 -n -e web/history

# List the configured providers and whether they support adding
cargo run -q -- providers

# Add an event to a provider
cargo run -q -- add --provider history --date 2026-05-12 \
    --description "Finished the Rust course" --category "personal/school"
```

Example output:

```
$ cargo run -q -- -d 0510 -n -e web/history
1972-05-10: A-10 Thunderbolt II takes its first flight at Edwards Air Force Base (military/aviation)
2025-05-10: Today I learned Rust (personal/learning)
1913-05-10: Igor Sikorsky's Bolshoi Baltiskiy becomes the world's first four-engine aircraft to fly (military/aviation)

$ cargo run -q -- providers
- myprovider (MyProvider) [read-only]
- history-db (sqlite) [read-only]
- history (text) [supports add]
- history-csv (csv) [supports add]
- muffinlabs (web) [read-only]
```

## Data formats

- **Text:** four lines per event: date (`YYYY-MM-DD`), description, category (`primary/secondary`) and an empty separator line
- **CSV:** header `date,description,category`, then one event per line
- **SQLite:** tables `event` and `category`

## Build and test

```bash
cargo build
cargo test
```

## Project structure

```
src/
├── main.rs            Command-line parsing (clap), calls the library
├── lib.rs             run(), list_providers(), add_event(), Config
├── birthday.rs        Age and birthday message
├── events.rs          Event, Category, MonthDay
├── filters.rs         EventFilter and FilterBuilder
└── providers/
    ├── mod.rs         EventProvider trait, EventProviderError
    ├── textfile.rs    Text file provider
    ├── csvfile.rs     CSV file provider
    ├── sqlite.rs      SQLite provider
    ├── web.rs         Web API provider
    └── myprovider.rs  Hard-coded provider used for testing
config-example/        Example today.toml and data files
```

Crates used: chrono, clap, csv, dirs, reqwest (blocking), serde, serde_json, sqlite, toml. Code comments are in Finnish.

## Limitations

- The web and SQLite providers are read-only.
- Events before year 1 from the web API are skipped.
- If the web API cannot be reached, the program prints an error and still shows events from the other providers.

## What I learned

- Trait objects and default trait methods: adding a new provider type needs changes in only two places
- Error handling with `Result`, `?` and a custom error type
- Splitting a program into a binary crate and a library crate, with unit tests per module
