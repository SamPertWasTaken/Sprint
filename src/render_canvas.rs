#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8
}
impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r, g, b, a
        }
    }
    pub fn new_mono(mono: u8, a: u8) -> Self {
        Self {
            r: mono, 
            g: mono, 
            b: mono, 
            a
        }
    }
    pub fn from_tuple(tuple: (u8, u8, u8), a: u8) -> Self {
        Self::new(tuple.0, tuple.1, tuple.2, a)
    }

    pub fn get_wayland_color(self) -> i32 {
        (i32::from(self.a) << 24) + (i32::from(self.r) << 16) + (i32::from(self.g) << 8) + i32::from(self.b)
    }
}

pub struct RenderCanvas {
    pixels: Vec<Color>,
    width: u32,
    height: u32
}
impl RenderCanvas {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            pixels: vec![Color::new(0, 0, 0, 255); (width * height) as usize],
            width,
            height
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x >= self.width || y >= self.height {
            return;
        }
        let index = self.index_from_pixel(x, y);
        self.pixels[index] = color;
    }
    pub fn draw_box(&mut self, x: u32, y: u32, w: u32, h: u32, color: Color) {
        for box_x in x..x + w {
            for box_y in y..y + h {
                self.set_pixel(box_x, box_y, color);
            }
        }
    }
    pub fn wipe(&mut self, color: Color) {
        self.pixels = vec![color; (self.width * self.height) as usize];
    }

    pub fn fill_wayland_canvas(&self, canvas: &mut [u8]) {
        canvas.chunks_exact_mut(4).enumerate().for_each(|(index, chunk)| {
            let width_usize = usize::try_from(self.width).expect("width to usize failed");
            let x = u32::try_from(index % width_usize).expect("x to u32 failed");
            let y = u32::try_from(index / width_usize).expect("y to u32 failed");

            let pixel_index: usize = self.index_from_pixel(x, y);
            let array: &mut [u8; 4] = chunk.try_into().unwrap();
            *array = self.pixels[pixel_index].get_wayland_color().to_le_bytes();
        });
    }

    fn index_from_pixel(&self, x: u32, y: u32) -> usize {
        (x + self.width * y) as usize
    }
}

pub trait CanvasRenderable {
    fn draw(&mut self, canvas: &mut RenderCanvas);
}
