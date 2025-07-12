#![warn(clippy::pedantic)]
#![warn(clippy::redundant_type_annotations)]

use sprint_config::SprintConfig;

mod entry_box;
mod input_box;
mod render_canvas;
mod results;
mod sprint_config;
mod text_label;
mod wayland;

fn main() {
    let config = SprintConfig::load();
    wayland::create_layer(config);
}
