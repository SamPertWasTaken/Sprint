use std::process::{Command, Stdio};

use font_kit::font::Font;
use freedesktop_desktop_entry::{get_languages_from_env, DesktopEntry};
use pathfinder_geometry::vector::Vector2I;

use crate::{render_canvas::CanvasRenderable, text_label::TextLabel};

#[derive(Debug)]
pub enum EntryBoxValue {
    Desktop(DesktopEntry),
    Math(f64),
    WebSearch(String, String),
    WebPrefix(String, String, String),
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Entrybox {
    value: EntryBoxValue,
    position: Vector2I,
    size: Vector2I,
    label: TextLabel
}

impl Entrybox {
    pub fn new(value: EntryBoxValue, position: Vector2I, size: Vector2I, font: Font) -> Self {
        let locales = get_languages_from_env();
        let label = match value {
            EntryBoxValue::Desktop(ref desktop_entry) => desktop_entry.full_name(&locales).expect("Failed to get desktop name").to_string(),
            EntryBoxValue::Math(math) => format!("= {math}"),
            EntryBoxValue::WebSearch(ref query, _) => format!("Search \"{query}\" on the web..."),
            EntryBoxValue::WebPrefix(ref name, ref query, _) => format!("Search \"{query}\" on \"{name}\"...")
        };
        Self {
            value,
            position,
            size,
            label: TextLabel::new(&label, font, 16.0, position, size)
        }
    }

    pub fn select(&self) {
        match &self.value {
            EntryBoxValue::Desktop(desktop_entry) => {
                println!("{}", desktop_entry.exec().unwrap());
                let exec = desktop_entry.exec().expect("Desktop entry does not contain an exec.");

                let command: String = exec.split_whitespace()
                    .filter(|x| !x.starts_with('%'))
                    .collect::<Vec<&str>>()
                    .join(" ");
                println!(" parsed to {command}");

                #[allow(clippy::zombie_processes)]
                Command::new("sh")
                    .args(["-c", &command])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .expect("Unable to launch process");
            },
            EntryBoxValue::Math(_) => {},
            EntryBoxValue::WebSearch(_, url) | EntryBoxValue::WebPrefix(_, _, url) => webbrowser::open(url).expect("Failed to launch url on web browser."),
        }
    }
}

impl CanvasRenderable for Entrybox {
    fn draw(&mut self, canvas: &mut crate::render_canvas::RenderCanvas) {
        self.label.draw(canvas);
    }
}
