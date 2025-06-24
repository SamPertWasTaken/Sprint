//! This module is responsible for the actual searching and calculating of the results.
//! Currently it will use the following;
//! - FreeDesktop .desktop entry files
//! - Calculations

use std::time::Instant;
use freedesktop_desktop_entry::{default_paths, get_languages_from_env, DesktopEntry, Iter};

use crate::sprint_config::SprintConfig;

pub fn return_results(input: &str, config: &SprintConfig) -> SprintResults {
    let time: Instant = Instant::now();
    let results = SprintResults {
        math_result: math(input),
        desktop_results: desktop_entries(input),
        web_results: web(input, config)
    };
    println!("Results search time for '{input}': {:?}", Instant::now() - time);
    results
}

fn math(input: &str) -> Option<f64> {
    if let Ok(r) = meval::eval_str(input) {
        return Some(r);
    }
    None
}

fn web(input: &str, config: &SprintConfig) -> (String, String) {
    // config.search_template.replace("%%QUERY%%", &input.replace(" ", "+"))
    (input.to_string(), config.search_template.replace("%%QUERY%%", &input.replace(" ", "+")))
}

fn desktop_entries(input: &str) -> Vec<DesktopEntry> {
    let locales = get_languages_from_env();

    let mut entries = Iter::new(default_paths())
        .entries(Some(&locales))
        .filter(|x| x.full_name(&locales).unwrap().to_lowercase().contains(&input.to_lowercase()))
        .collect::<Vec<_>>();
    entries.sort_unstable_by_key(|item| item.full_name(&locales).expect("Failed to fetch app name from locale.").to_string());
    entries
}

#[derive(Default, Debug)]
pub struct SprintResults {
    pub math_result: Option<f64>,
    pub desktop_results: Vec<DesktopEntry>,
    pub web_results: (String, String)
}
