//! This module is responsible for the actual searching and calculating of the results.
//! Currently it will use the following;
//! - FreeDesktop .desktop entry files
//! - Calculations

use std::time::Instant;
use freedesktop_desktop_entry::{current_desktop, default_paths, get_languages_from_env, DesktopEntry, Iter};

use crate::sprint_config::SprintConfig;

pub fn return_results(input: &str, config: &SprintConfig) -> SprintResults {
    let time: Instant = Instant::now();
    let results = SprintResults {
        prefix_results: web_prefixes(input, config),
        math_result: math(input),
        desktop_results: desktop_entries(input),
        web_results: web(input, config)
    };
    println!("Results search time for '{input}': {:?}", Instant::now() - time);
    results
}

fn web_prefixes(input: &str, config: &SprintConfig) -> Option<(String, String, String)> {
    let mut result: Option<(String, String, String)> = None;
    for prefix in &config.web_prefixes {
        if let Some(query) = input.strip_prefix(&prefix.1) {
            if result.is_some() {
                // too many matches
                return None;
            }
            let mut prefix = prefix.clone();
            prefix.1 = query.trim().to_string();
            prefix.2 = prefix.2.replace("%%QUERY%%", &query.trim().replace(" ", "+"));
            result = Some(prefix.clone());
        }
    }

    result
}

fn math(input: &str) -> Option<f64> {
    if let Ok(r) = meval::eval_str(input) {
        return Some(r);
    }
    None
}

fn web(input: &str, config: &SprintConfig) -> (String, String) {
    (input.to_string(), config.search_template.replace("%%QUERY%%", &input.replace(" ", "+")))
}

fn desktop_entries(input: &str) -> Vec<DesktopEntry> {
    let locales = get_languages_from_env();
    let current_desktop = current_desktop().expect("Failed to find the current desktop.");

    let mut entries = Iter::new(default_paths())
        .entries(Some(&locales))
        .filter(|entry| entry.full_name(&locales).unwrap().to_lowercase().contains(&input.to_lowercase()))
        .filter(|entry| !entry.no_display())
        .filter(|entry| {
            if let Some(show_in) = entry.only_show_in() {
                return show_in.iter().any(|x| current_desktop.contains(&x.to_string()));
            }
            true
        })
        .filter(|entry| {
            if let Some(no_show_in) = entry.not_show_in() {
                return !no_show_in.iter().any(|x| current_desktop.contains(&x.to_string()));
            }
            true
        })
        .collect::<Vec<_>>();
    
    entries.sort_unstable_by_key(|item| item.full_name(&locales).expect("Failed to fetch app name from locale.").to_string());
    entries
}

#[derive(Default, Debug)]
pub struct SprintResults {
    pub prefix_results: Option<(String, String, String)>,
    pub math_result: Option<f64>,
    pub desktop_results: Vec<DesktopEntry>,
    pub web_results: (String, String)
}
