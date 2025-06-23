use std::cmp::{max, min};

use font_kit::font::Font;
use pathfinder_geometry::vector::Vector2I;

use crate::{render_canvas::{CanvasRenderable, Color}, text_label::TextLabel};

pub struct InputBox {
    position: Vector2I,
    size: Vector2I,
    placeholder: String,
    text: String,
    cursor_pos: usize,

    label: TextLabel,
    placeholder_label: TextLabel
}

impl InputBox {
    pub fn new(starting_text: &str, placeholder: &str, position: Vector2I, size: Vector2I, font: Font) -> Self {
        Self {
            position,
            size,
            placeholder: placeholder.to_string(),
            text: starting_text.to_string(),
            cursor_pos: 0,
            label: TextLabel::new(starting_text, font.clone(), 18.0, position, size).expect("Failed to create input box label."),
            placeholder_label: TextLabel::new(placeholder, font.clone(), 18.0, position, size).expect("Failed to create input box placeholder label.")
        }
    }

    pub fn set_cursor_pos(&mut self, pos: usize) {
        self.cursor_pos = max(min(pos, self.text.chars().count()), 0);
    }
    pub fn advance_cursor(&mut self) {
        self.set_cursor_pos(self.cursor_pos + 1);
    }
    pub fn reel_cursor(&mut self) {
        self.set_cursor_pos(self.cursor_pos - 1);
    }

    pub fn set_text(&mut self, new_text: &str) {
        self.text = new_text.to_string();
        self.label.set_text(new_text);
    }

    pub fn push_at_cursor(&mut self, ch: char) -> String {
        self.text.insert(self.cursor_pos, ch);
        self.label.set_text(&self.text);
        self.cursor_pos += 1;

        self.text.to_string()
    }
    pub fn pop_at_cursor(&mut self) -> Option<String> {
        if self.text.is_empty() || self.cursor_pos == 0 {
            return None;
        }
        self.text.remove(self.cursor_pos - 1);
        self.label.set_text(&self.text);
        self.cursor_pos -= 1;
        Some(self.text.to_string())
    }
}
impl CanvasRenderable for InputBox {
    fn draw(&mut self, canvas: &mut crate::render_canvas::RenderCanvas) {
        if self.text.is_empty() {
            self.placeholder_label.draw(canvas);
        } else {
            self.label.draw(canvas);
        }

        // the label will have updated it's bounds cache by this point so we don't need to worry
        // about calling it explicitly
        canvas.draw_box(self.position.x() as u32 + (self.label.find_cursor_length(self.cursor_pos)), self.position.y() as u32, 1, self.size.y() as u32, Color::new_mono(255, 255));
    }
}
