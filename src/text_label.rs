use std::collections::HashMap;

use font_kit::{canvas::{Canvas, Format, RasterizationOptions}, font::Font, hinting::HintingOptions, source::SystemSource};
use pathfinder_geometry::{rect::RectI, transform2d::Transform2F, vector::{Vector2F, Vector2I}};

use crate::render_canvas::{CanvasRenderable, Color};

#[derive(Debug)]
pub struct TextLabel {
    position: Vector2I,
    size: Vector2I,
    font_canvas: Option<Canvas>,
    text: String,
    font: Font,
    font_size: f32,
    requires_rerender: bool,
    character_length_cache: HashMap<char, RectI>
}

impl TextLabel {
    pub fn new(text: &str, font: Font, font_size: f32, position: Vector2I, size: Vector2I) -> Option<Self> {
        Some(Self {
            position,
            size,
            font_canvas: None,
            requires_rerender: true, // triggers the first render
            text: text.to_string(),
            font,
            font_size,
            character_length_cache: HashMap::new()
        })
    }
    
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.requires_rerender = true;
    }

    pub fn find_cursor_length(&self, place: usize) -> u32 {
        let mut length: u32 = 0;
        self.text[0..place]
            .chars()
            .for_each(|char| length += self.character_length_cache[&char].width() as u32);
        length
    }

    fn rasterize_to_font_canvas(&mut self) {
        if !self.requires_rerender {
            return;
        }

        self.font_canvas = Some(Canvas::new(self.size, Format::A8));

        self.character_length_cache = HashMap::new();
        let canvas = self.font_canvas.as_mut().unwrap();

        let mut transform: Transform2F = Transform2F::from_translation(Vector2F::new(0.0, self.size.y() as f32 / 1.5));
        for char in self.text.chars() {
            if char.is_whitespace() {
                // transform and move on 
                transform = transform.translate(Vector2F::new(8.0, 0.0));
                self.character_length_cache.entry(' ').or_insert(RectI::new(Vector2I::new(0, 0), Vector2I::new(8, 0)));
                continue;
            }

            let mut glyph_id = 0; // unknown glyph
            if let Some(found_id) = self.font.glyph_for_char(char) {
                glyph_id = found_id;
            }
            // find the bounds so we can transform the next char correctly
            let bounds = self.font.raster_bounds(glyph_id, self.font_size, transform, HintingOptions::None, RasterizationOptions::GrayscaleAa).unwrap();
            self.character_length_cache.entry(char).or_insert(bounds);
            // actually render it to the canvas
            self.font.rasterize_glyph(canvas, glyph_id, self.font_size, transform, HintingOptions::None, RasterizationOptions::GrayscaleAa).unwrap();
            // adjust the transform
            transform = transform.translate(Vector2F::new(bounds.width() as f32, 0.0));
        }
        self.requires_rerender = false;
    }

    fn find_font(source: &SystemSource, postscript_name: &str) -> Option<Font> {
        if let Ok(font) = source.select_by_postscript_name(postscript_name) {
            match font.load() {
                Ok(r) => return Some(r),
                Err(_) => return None,
            }
        }
        None
    }
}
impl CanvasRenderable for TextLabel {
    fn draw(&mut self, canvas: &mut crate::render_canvas::RenderCanvas) {
        // canvas.draw_box(self.position.x() as u32, self.position.y() as u32, self.size.x() as u32, self.size.y() as u32, Color::new(255, 0, 0, 255));
        self.rasterize_to_font_canvas();
        if self.font_canvas.is_none() {
            return;
        }

        let font_canvas = self.font_canvas.as_ref().unwrap();
        for y in 0..self.size.y() {
            for x in 0..self.size.x() {
                let final_x: u32 = (x + self.position.x()) as u32;
                let final_y = (y + self.position.y()) as u32;

                let row = font_canvas.stride * y as usize;
                let pixel_index = row + (font_canvas.format.bytes_per_pixel() as usize * x as usize);
                let color = font_canvas.pixels[pixel_index];
                if color == 0 {
                    continue;
                }
                canvas.set_pixel(final_x, final_y, Color::new_mono(color, 255));
            }
        }
    }
}
