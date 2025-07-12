use std::{env, fs, io::Write, path::PathBuf};

use config::{Config, File};
use font_kit::{font::Font, source::SystemSource};
use serde::Deserialize;

use crate::render_canvas::Color;

const DEFAULT_CONFIG_CONTENTS: &str = include_str!("../default-config.toml");

#[derive(Clone, Debug, Deserialize)]
struct SprintConfigRaw {
    font: String,
    background_color: (u8, u8, u8),
    foreground_color: (u8, u8, u8),
    seperator_color: (u8, u8, u8),
    selection_hover_color: (u8, u8, u8),
    search_template: String,
    web_prefixes: Vec<(String, String, String)>,
    result_order: Vec<String>
}
impl Default for SprintConfigRaw {
    fn default() -> Self {
        Self {
            font: "FreeSans".to_string(),
            background_color: (25, 25, 25),
            foreground_color: (30, 30, 30),
            seperator_color: (112, 69, 156),
            selection_hover_color: (72, 43, 102),
            search_template: "https://duckduckgo.com/?q=%%QUERY%%".to_string(),
            web_prefixes: vec![
                ("Wikipedia".to_string(), ">wiki".to_string(), "https://en.wikipedia.org/w/index.php?search=%%QUERY%%".to_string()),
                ("StackExchange".to_string(), ">exchange".to_string(), "https://stackexchange.com/search?q=%%QUERY%%".to_string()),
                ("StackOverflow".to_string(), ">overflow".to_string(), "https://stackoverflow.com/search?q=%%QUERY%%".to_string()),

                ("YouTube".to_string(), ">yt".to_string(), "https://www.youtube.com/results?search_query=%%QUERY%%".to_string()),
                ("GitHub".to_string(), ">gh".to_string(), "https://github.com/search?q=%%QUERY%%".to_string()),
                ("LinkedIn".to_string(), ">lnkin".to_string(), "https://www.linkedin.com/search/results/all/?keywords=%%QUERY%%".to_string()),
                ("Reddit".to_string(), ">reddit".to_string(), "https://www.reddit.com/search/?q=%%QUERY%%".to_string()),
                ("Facebook".to_string(), ">facebook".to_string(), "https://www.facebook.com/search/top/?q=%%QUERY%%".to_string()),

                ("Google".to_string(), ">google".to_string(), "https://www.google.com/search?q=%%QUERY%%".to_string()),
                ("Bing".to_string(), ">bing".to_string(), "https://www.bing.com/search?q=%%QUERY%%".to_string()),
                ("DuckDuckGo".to_string(), ">ddg".to_string(), "https://duckduckgo.com/?q=%%QUERY%%".to_string()),
            ],
            result_order: vec!["prefixes".to_string(), "math".to_string(), "desktop".to_string(), "search".to_string()]
        }
    }
}
impl SprintConfigRaw {
    pub fn load() -> Self {
        if let Some(config) = SprintConfigRaw::locate_config() {
            Config::builder()
                .add_source(File::from(config))
                .build()
                .expect("Failed to load configuration.")
                .try_deserialize::<SprintConfigRaw>()
                .expect("Failed to deserialize config file.")
        } else {
            SprintConfigRaw::default()
        }
    }

    fn locate_config() -> Option<PathBuf> {
        if let Ok(mut config_home) = env::var("XDG_CONFIG_HOME") {
            config_home.push_str("/sprint.toml");
            let path = PathBuf::from(config_home);
            if path.exists() {
                return Some(path);
            } 

            Self::generate_default_config_file(path);
            return None;
        }
        if let Ok(mut user_home) = env::var("HOME") {
            user_home.push_str("/.config/sprint.toml");
            let path = PathBuf::from(user_home);
            if path.exists() {
                return Some(path);
            }

            Self::generate_default_config_file(path);
            return None;
        }

        None
    }

    fn generate_default_config_file(path: PathBuf) {
        if path.exists() {
            return;
        }

        let mut config_file = fs::File::create(&path).expect("Unable to create default config file.");
        config_file.write_all(DEFAULT_CONFIG_CONTENTS.as_bytes()).expect("Unable to write to default config file.");
        println!("Created default config file");
    }
}

#[derive(Clone, Debug)]
pub struct SprintConfig {
    raw: SprintConfigRaw,
    // TODO: Currently fonts have to be cloned due to it not impling copy, is there a way around
    // this? Got close with Cow's but tainting every struct with a lifetime seems
    // counter-productive
    pub font: Font,
    pub background_color: Color,
    pub foreground_color: Color,
    pub seperator_color: Color,
    pub selection_hover_color: Color,
    pub search_template: String,
    pub web_prefixes: Vec<(String, String, String)>,
    pub result_order: Vec<String>
}
impl SprintConfig {
    pub fn load() -> Self {
        let raw_config = SprintConfigRaw::load();

        // Load the font
        let font_source = SystemSource::new();
        let font_handle = font_source.select_by_postscript_name(&raw_config.font).expect("Failed to find font for configuration.");
        let font = font_handle.load().expect("Failed to load font.");

        Self {
            font,
            background_color: Color::from_tuple(raw_config.background_color, 255),
            foreground_color: Color::from_tuple(raw_config.foreground_color, 255),
            seperator_color: Color::from_tuple(raw_config.seperator_color, 255),
            selection_hover_color: Color::from_tuple(raw_config.selection_hover_color, 255),
            search_template: raw_config.search_template.to_string(),
            web_prefixes: raw_config.web_prefixes.clone(),
            result_order: raw_config.result_order.clone(),

            raw: raw_config
        }
    }
}
