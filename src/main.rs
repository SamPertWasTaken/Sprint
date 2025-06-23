mod entry_box;
mod input_box;
mod render_canvas;
mod results;
mod text_label;
mod wayland;

const FONT: &str = "TitilliumWeb-Regular";

fn main() {
    wayland::create_layer();
}

