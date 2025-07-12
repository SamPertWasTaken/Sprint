//! This module is responsible for the actual searching and calculating of the results.
//! Currently it will use the following;
//! - FreeDesktop .desktop entry files
//! - Calculations

use std::time::Instant;
use freedesktop_desktop_entry::{current_desktop, default_paths, get_languages_from_env, DesktopEntry, Iter};

use crate::sprint_config::SprintConfig;

#[derive(Default, Debug)]
pub struct SprintResults {
    pub prefix_results: Option<(String, String, String)>,
    pub math_result: Option<f64>,
    pub desktop_results: Vec<DesktopEntry>,
    pub web_result: (String, String),

    desktop_file_cache: Vec<DesktopEntry>,
    desktop_locale_cache: Vec<String>,
    current_desktop: Option<Vec<String>>
}
impl SprintResults {
    pub fn new() -> Self {
        let locales = get_languages_from_env();

        Self {
            prefix_results: None,
            math_result: None,
            desktop_results: Vec::new(),
            web_result: ("".to_string(), "".to_string()),

            desktop_file_cache: Iter::new(default_paths())
                .entries(Some(&locales))
                .collect::<Vec<_>>(),
            desktop_locale_cache: locales,
            current_desktop: current_desktop()
        }
    }

    pub fn refresh_results(&mut self, input: &str, config: &SprintConfig) {
        let time: Instant = Instant::now();

        self.prefix_results = Self::get_prefix_results(input, config);
        self.math_result = Self::get_math_result(input);
        self.desktop_results = Self::get_desktop_entries(input, &self.desktop_file_cache, &self.desktop_locale_cache, &self.current_desktop);
        self.web_result = Self::get_web_result(input, config);

        println!("Results search time for '{input}': {:?}", Instant::now() - time);
    }

    fn get_prefix_results(input: &str, config: &SprintConfig) -> Option<(String, String, String)> {
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

    fn get_math_result(input: &str) -> Option<f64> {
        if let Ok(r) = meval::eval_str(input) {
            return Some(r);
        }
        None
    }

    fn get_web_result(input: &str, config: &SprintConfig) -> (String, String) {
        (input.to_string(), config.search_template.replace("%%QUERY%%", &input.replace(" ", "+")))
    }

    fn get_desktop_entries(input: &str, desktop_files: &Vec<DesktopEntry>, desktop_locales: &Vec<String>, current_desktop: &Option<Vec<String>>) -> Vec<DesktopEntry> {
        let mut entries = desktop_files.iter()
            // Name
            .filter(|entry| entry.full_name(&desktop_locales).unwrap().to_lowercase().contains(&input.to_lowercase()))
            // Is it hidden?
            .filter(|entry| !entry.no_display())
            // Only show in these desktops
            .filter(|entry| {
                if let Some(current_desktop) = &current_desktop {
                    if let Some(show_in) = entry.only_show_in() {
                        return show_in.iter().any(|x| current_desktop.contains(&x.to_string()));
                    }
                    return true;
                }
                true
            })
            // Do not show in these desktops
            .filter(|entry| {
                if let Some(current_desktop) = &current_desktop {
                    if let Some(no_show_in) = entry.not_show_in() {
                        return !no_show_in.iter().any(|x| current_desktop.contains(&x.to_string()));
                    }
                    return true;
                }
                true
            })
            // TODO: there are a million better ways to do this...
            .map(|x| x.to_owned())
            .collect::<Vec<_>>();

        entries.sort_unstable_by_key(|item| item.full_name(&desktop_locales).expect("Failed to fetch app name from locale.").to_string());
        entries
    }
}
