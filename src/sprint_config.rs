use std::{env, path::PathBuf};

use config::{Config, File};
use font_kit::{font::Font, source::SystemSource};
use serde::Deserialize;

use crate::render_canvas::Color;

#[derive(Debug, Deserialize)]
struct SprintConfigRaw {
    font: String,
    background_color: (u8, u8, u8),
    foreground_color: (u8, u8, u8),
    seperator_color: (u8, u8, u8),
    selection_hover_color: (u8, u8, u8),
    search_template: String
}
impl Default for SprintConfigRaw {
    fn default() -> Self {
        Self {
            font: "FreeSans".to_string(),
            background_color: (25, 25, 25),
            foreground_color: (30, 30, 30),
            seperator_color: (112, 69, 156),
            selection_hover_color: (72, 43, 102),
            search_template: "https://duckduckgo.com/?q=%%QUERY%%".to_string()
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
        }
        if let Ok(mut user_home) = env::var("HOME") {
            user_home.push_str("/.config/sprint.toml");
            let path = PathBuf::from(user_home);
            if path.exists() {
                return Some(path);
            }
        }

        None
    }
}

#[derive(Debug)]
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
    pub search_template: String
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

            raw: raw_config
        }
    }
}
