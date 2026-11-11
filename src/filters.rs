use std::collections::HashSet;
use crate::events::{Event, MonthDay, Category};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FilterOption {
    MonthDay(MonthDay),
    Category(Category),
    Text(String),
    ExcludedCategory(Category),
}

pub struct EventFilter {
    options: HashSet<FilterOption>
}

impl EventFilter {
    pub fn new() -> Self {
        Self {
            options: HashSet::new(),
        }
    }

    pub fn accepts(&self, event: &Event) -> bool {
        // Käsitellään ekskluusio ensin: jos tapahtuman kategoria
        // on missä tahansa ExcludedCategory-asetuksessa, hylätään se heti
        for option in self.options.iter() {
            if let FilterOption::ExcludedCategory(cat) = option {
                if *cat == event.category() {
                    return false;
                }
            }
        }

        // Inkluusiosäännöt: tapahtuma hyväksytään jos KAIKKI annetut
        // ehdot täyttyvät
        if self.options.is_empty() {
            return true;
        }

        let mut has_inclusive_rule = false;
        let mut results: Vec<bool> = Vec::new();

        for option in self.options.iter() {
            let result = match option {
                FilterOption::MonthDay(month_day) => {
                    has_inclusive_rule = true;
                    *month_day == event.month_day()
                },
                FilterOption::Category(category) => {
                    has_inclusive_rule = true;
                    *category == event.category()
                },
                FilterOption::Text(text) => {
                    has_inclusive_rule = true;
                    event.description().contains(text.as_str())
                },
                FilterOption::ExcludedCategory(_) => {
                    // Ekskluusio on jo käsitelty yllä, ei vaikuta tähän
                    continue;
                },
            };
            results.push(result);
        }

        if !has_inclusive_rule {
            return true;
        }

        results.iter().all(|&r| r)
    }

    pub fn contains_month_day(&self) -> bool {
        self.options.iter().any(|o| matches!(o, FilterOption::MonthDay(_)))
    }

    pub fn contains_category(&self) -> bool {
        self.options.iter().any(|o| matches!(o, FilterOption::Category(_)))
    }

    pub fn contains_text(&self) -> bool {
        self.options.iter().any(|o| matches!(o, FilterOption::Text(_)))
    }

    pub fn month_day(&self) -> Option<MonthDay> {
        for option in self.options.iter() {
            if let FilterOption::MonthDay(md) = option {
                return Some(md.clone());
            }
        }
        None
    }
}

pub struct FilterBuilder {
    options: HashSet<FilterOption>,
}

impl FilterBuilder {
    pub fn new() -> FilterBuilder {
        FilterBuilder {
            options: HashSet::new(),
        }
    }

    pub fn month_day(mut self, month_day: MonthDay) -> FilterBuilder {
        self.options.insert(FilterOption::MonthDay(month_day));
        self
    }

    pub fn category(mut self, category: Category) -> FilterBuilder {
        self.options.insert(FilterOption::Category(category));
        self
    }

    pub fn text(mut self, text: String) -> FilterBuilder {
        self.options.insert(FilterOption::Text(text));
        self
    }

    pub fn exclude_category(mut self, category: Category) -> FilterBuilder {
        self.options.insert(FilterOption::ExcludedCategory(category));
        self
    }

    pub fn build(self) -> EventFilter {
        EventFilter {
            options: self.options,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn make_event(date: NaiveDate, description: &str, category_str: &str) -> Event {
        Event::new_singular(date, description.to_string(), Category::from_primary(category_str))
    }

    #[test]
    fn filter_accepts_anything_when_empty() {
        let event = make_event(
            NaiveDate::from_ymd_opt(2026, 3, 26).unwrap(),
            "Some event",
            "misc"
        );
        let filter = FilterBuilder::new().build();
        assert!(filter.accepts(&event));
    }

    #[test]
    fn filter_accepts_correct_month_day() {
        let event = make_event(
            NaiveDate::from_ymd_opt(2026, 3, 26).unwrap(),
            "Some event",
            "misc"
        );
        let filter = FilterBuilder::new()
            .month_day(MonthDay::new(3, 26))
            .build();
        assert!(filter.accepts(&event));
    }

    #[test]
    fn filter_rejects_wrong_month_day() {
        let event = make_event(
            NaiveDate::from_ymd_opt(2026, 3, 26).unwrap(),
            "Some event",
            "misc"
        );
        let filter = FilterBuilder::new()
            .month_day(MonthDay::new(12, 11))
            .build();
        assert!(!filter.accepts(&event));
    }

    #[test]
    fn filter_accepts_correct_category() {
        let event = make_event(
            NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
            "Scream 4 released",
            "horror"
        );
        let filter = FilterBuilder::new()
            .category(Category::from_primary("horror"))
            .build();
        assert!(filter.accepts(&event));
    }

    #[test]
    fn filter_rejects_wrong_category() {
        let event = make_event(
            NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
            "Scream 4 released",
            "horror"
        );
        let filter = FilterBuilder::new()
            .category(Category::from_primary("sports"))
            .build();
        assert!(!filter.accepts(&event));
    }

    #[test]
    fn filter_accepts_matching_text() {
        let event = make_event(
            NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
            "Scream 4 released",
            "horror"
        );
        let filter = FilterBuilder::new()
            .text("released".to_string())
            .build();
        assert!(filter.accepts(&event));
    }

    #[test]
    fn filter_rejects_missing_text() {
        let event = make_event(
            NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
            "Scream 4 released",
            "horror"
        );
        let filter = FilterBuilder::new()
            .text("Wes Craven".to_string())
            .build();
        assert!(!filter.accepts(&event));
    }

    #[test]
    fn filter_accepts_when_all_criteria_match() {
        let event = make_event(
            NaiveDate::from_ymd_opt(2026, 12, 11).unwrap(),
            "Death of Wes Craven",
            "horror"
        );
        let filter = FilterBuilder::new()
            .month_day(MonthDay::new(12, 11))
            .category(Category::from_primary("horror"))
            .text("Wes".to_string())
            .build();
        assert!(filter.accepts(&event));
    }

    #[test]
    fn filter_rejects_when_one_criterion_fails() {
        let event = make_event(
            NaiveDate::from_ymd_opt(2026, 12, 11).unwrap(),
            "Death of Wes Craven",
            "horror"
        );
        let filter = FilterBuilder::new()
            .month_day(MonthDay::new(12, 11))
            .category(Category::from_primary("sports"))
            .text("Wes".to_string())
            .build();
        assert!(!filter.accepts(&event));
    }

    #[test]
    fn filter_excludes_listed_category() {
        let event = make_event(
            NaiveDate::from_ymd_opt(2026, 5, 10).unwrap(),
            "Some web event",
            "web"
        );
        let filter = FilterBuilder::new()
            .exclude_category(Category::from_primary("web"))
            .build();
        assert!(!filter.accepts(&event));
    }

    #[test]
    fn filter_accepts_non_excluded_category() {
        let event = make_event(
            NaiveDate::from_ymd_opt(2026, 5, 10).unwrap(),
            "Rally event",
            "rally"
        );
        let filter = FilterBuilder::new()
            .exclude_category(Category::from_primary("web"))
            .build();
        assert!(filter.accepts(&event));
    }

    #[test]
    fn filter_excludes_secondary_category() {
        let event = make_event(
            NaiveDate::from_ymd_opt(2026, 5, 10).unwrap(),
            "WRC event",
            "rally"
        );
        // Käytetään täyttä kategoriaa primary/secondary
        let full_category = Category::new("rally", "wrc");
        let event_full = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 5, 10).unwrap(),
            "WRC win".to_string(),
            full_category.clone(),
        );
        let filter = FilterBuilder::new()
            .exclude_category(full_category)
            .build();
        assert!(!filter.accepts(&event_full));
        // Pelkkä "rally" ei sulje pois "rally/wrc":a tai päinvastoin
        // koska Category-vertailu vaatii sekä primaryn että secondaryn täsmäämistä
        let filter_primary_only = FilterBuilder::new()
            .exclude_category(Category::from_primary("rally"))
            .build();
        assert!(filter_primary_only.accepts(&event_full));
        assert!(!filter_primary_only.accepts(&event)); // tapahtuma jolla vain primary "rally"
    }

    #[test]
    fn filter_exclude_combines_with_month_day() {
        // Tapahtuma 10.5. kategoriassa "web"
        let excluded_event = make_event(
            NaiveDate::from_ymd_opt(2026, 5, 10).unwrap(),
            "Web event",
            "web"
        );
        // Tapahtuma 10.5. kategoriassa "rally"
        let kept_event = make_event(
            NaiveDate::from_ymd_opt(2026, 5, 10).unwrap(),
            "Rally event",
            "rally"
        );
        // Tapahtuma 11.5. kategoriassa "rally"
        let wrong_day_event = make_event(
            NaiveDate::from_ymd_opt(2026, 5, 11).unwrap(),
            "Wrong day",
            "rally"
        );

        let filter = FilterBuilder::new()
            .month_day(MonthDay::new(5, 10))
            .exclude_category(Category::from_primary("web"))
            .build();

        assert!(!filter.accepts(&excluded_event));   // suljettu ekskluusion takia
        assert!(filter.accepts(&kept_event));          // päivä täsmää, ei sulkua
        assert!(!filter.accepts(&wrong_day_event));    // päivä ei täsmää
    }
}