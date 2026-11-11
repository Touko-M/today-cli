use std::fs;
use std::path::PathBuf;
use today::{run, list_providers, add_event, Config};
use today::events::{Category, MonthDay};
use dirs;
use clap::{Parser, Subcommand};
use chrono::NaiveDate;

#[derive(Parser)]
#[command(name = "today")]
#[command(about = "Show historical events for the current day")]
struct Args {
    /// Event date in MMDD format
    #[arg(short = 'd', long = "date")]
    date: Option<String>,

    /// Categories to exclude, comma-separated (a/b,c/d)
    #[arg(short = 'e', long = "exclude")]
    exclude: Option<String>,

    /// No age calculation or birthday message
    #[arg(short = 'n', long = "no-birthday")]
    no_birthday: bool,

    #[command(subcommand)]
    cmd: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// List all event providers
    Providers,

    /// Adds an event to an event provider
    Add {
        /// Name of event provider
        #[arg(short = 'p', long = "provider")]
        provider: String,

        /// Date of event. Format: YYYY-MM-DD
        #[arg(short = 'd', long = "date")]
        date: String,

        /// Description of event
        #[arg(short = 'e', long = "description")]
        description: String,

        /// Category of event. Format: primary[/secondary]
        #[arg(short = 'c', long = "category")]
        category: String,
    },
}

const APP_NAME: &str = "today";

fn get_config_path(app_name: &str) -> Option<PathBuf> {
    if let Some(config_dir) = dirs::config_dir() {
        let config_path = config_dir.join(app_name);
        if !config_path.exists() {
            if let Err(_) = fs::create_dir(&config_path) {
                eprintln!("Unable to create config directory for {}", app_name);
                return None;
            }
        }
        return Some(config_path);
    }
    None
}

fn load_config(config_path: &PathBuf) -> Config {
    let toml_path = config_path.join(format!("{}.toml", APP_NAME));
    let config_str = fs::read_to_string(&toml_path).expect("existing configuration file");
    toml::from_str(&config_str).expect("valid configuration file")
}

// Pilkulla erotettu lista kategorioita -> Vec<Category>
fn parse_exclude_list(s: &str) -> Vec<Category> {
    s.split(',')
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .map(|item| Category::from_str(item))
        .collect()
}

fn main() {
    let args = Args::parse();

    let config_path = match get_config_path(APP_NAME) {
        Some(p) => p,
        None => {
            eprintln!("Could not find or create configuration directory");
            return;
        }
    };

    let config = load_config(&config_path);

    // Alikomento valitsee koko ohjelman toimintaa
    match args.cmd {
        Some(Command::Providers) => {
            list_providers(&config, &config_path);
        }
        Some(Command::Add { provider, date, description, category }) => {
            // Parsitaan päivämäärä YYYY-MM-DD muodossa
            let parsed_date = match NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Invalid date '{}': {}", date, e);
                    return;
                }
            };
            let parsed_category = Category::from_str(&category);

            if let Err(e) = add_event(
                &config,
                &config_path,
                &provider,
                parsed_date,
                description,
                parsed_category,
            ) {
                eprintln!("Error: {}", e);
            }
        }
        None => {
            // Ei alikomentoa -> näytetään päivän tapahtumat
            let date_filter = match args.date {
                Some(date_str) => match MonthDay::parse_from_str(&date_str) {
                    Ok(month_day) => Some(month_day),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return;
                    }
                },
                None => None,
            };

            let exclude_list = match args.exclude {
                Some(s) => parse_exclude_list(&s),
                None => Vec::new(),
            };

            if let Err(e) = run(
                &config,
                &date_filter,
                &config_path,
                args.no_birthday,
                &exclude_list,
            ) {
                eprintln!("Virhe: {}", e);
            }
        }
    }
}