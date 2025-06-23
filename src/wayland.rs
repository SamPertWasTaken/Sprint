use std::{cmp::min, time::Instant};

use font_kit::source::SystemSource;
use pathfinder_geometry::vector::Vector2I;
use smithay_client_toolkit::{compositor::{CompositorHandler, CompositorState}, delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, delegate_registry, delegate_seat, delegate_shm, output::{OutputHandler, OutputState}, registry::{ProvidesRegistryState, RegistryState}, registry_handlers, seat::{keyboard::{KeyboardHandler, Keysym}, Capability, SeatHandler, SeatState}, shell::{wlr_layer::{KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface}, WaylandSurface}, shm::{slot::SlotPool, Shm, ShmHandler}};
use wayland_client::{globals::registry_queue_init, protocol::{wl_keyboard::WlKeyboard, wl_shm}, Connection, QueueHandle};

use crate::{entry_box::{EntryBoxValue, Entrybox}, input_box::InputBox, render_canvas::{CanvasRenderable, Color, RenderCanvas}, results::{self, SprintResults}, sprint_config::SprintConfig, text_label::TextLabel};

struct LayerState {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,
    close: bool,
    first_config: bool,
    pool: SlotPool,
    width: u32,
    height: u32,
    layer: LayerSurface,
    keyboard: Option<WlKeyboard>,
    canvas: RenderCanvas,

    // App Data
    config: SprintConfig,
    filter: String,
    filter_results: SprintResults,
    selected: u8,

    // Components
    filter_input: InputBox,
    filter_results_cache: Vec<Entrybox>,
    no_results_label: TextLabel
}

impl CompositorHandler for LayerState {
    fn frame(&mut self, _conn: &wayland_client::Connection, qh: &wayland_client::QueueHandle<Self>, _surface: &wayland_client::protocol::wl_surface::WlSurface, _time: u32) {
        self.draw(qh);
    }

    fn scale_factor_changed(&mut self, _conn: &wayland_client::Connection, _qh: &wayland_client::QueueHandle<Self>, _surface: &wayland_client::protocol::wl_surface::WlSurface, _new_factor: i32) {}
    fn transform_changed(&mut self, _conn: &wayland_client::Connection, _qh: &wayland_client::QueueHandle<Self>, _surface: &wayland_client::protocol::wl_surface::WlSurface, _new_transform: wayland_client::protocol::wl_output::Transform) {}
    fn surface_enter(&mut self, _conn: &wayland_client::Connection, _qh: &wayland_client::QueueHandle<Self>, _surface: &wayland_client::protocol::wl_surface::WlSurface, _output: &wayland_client::protocol::wl_output::WlOutput) {}
    fn surface_leave(&mut self, _conn: &wayland_client::Connection, _qh: &wayland_client::QueueHandle<Self>, _surface: &wayland_client::protocol::wl_surface::WlSurface, _output: &wayland_client::protocol::wl_output::WlOutput) {}
}

impl OutputHandler for LayerState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(&mut self, _conn: &wayland_client::Connection, _qh: &QueueHandle<Self>, _output: wayland_client::protocol::wl_output::WlOutput) {}
    fn update_output(&mut self, _conn: &wayland_client::Connection, _qh: &QueueHandle<Self>, _output: wayland_client::protocol::wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _conn: &wayland_client::Connection, _qh: &QueueHandle<Self>, _output: wayland_client::protocol::wl_output::WlOutput) {}
}

impl LayerShellHandler for LayerState {
    fn closed(&mut self, _conn: &wayland_client::Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.close = true;
    }

    fn configure(&mut self, _conn: &wayland_client::Connection, qh: &QueueHandle<Self>, _layer: &LayerSurface, configure: smithay_client_toolkit::shell::wlr_layer::LayerSurfaceConfigure, _serial: u32) {
        self.width = configure.new_size.0;
        self.height = configure.new_size.1;

        if self.first_config {
            self.first_config = false;
            self.draw(qh);
        }
    }
}

impl SeatHandler for LayerState {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_capability(&mut self, _conn: &wayland_client::Connection, qh: &QueueHandle<Self>, seat: wayland_client::protocol::wl_seat::WlSeat, capability: smithay_client_toolkit::seat::Capability) {
        if capability == Capability::Keyboard && self.keyboard.is_none() {
            let keyboard = self.seat_state.get_keyboard(qh, &seat, None).expect("Failed to create keyboard");
            self.keyboard = Some(keyboard);
        }
    }

    fn remove_capability(&mut self, _conn: &wayland_client::Connection, _qh: &QueueHandle<Self>, _seat: wayland_client::protocol::wl_seat::WlSeat, capability: smithay_client_toolkit::seat::Capability) {
        if capability == Capability::Keyboard && self.keyboard.is_some() {
            self.keyboard.take().unwrap().release();
        }
    }

    fn new_seat(&mut self, _conn: &wayland_client::Connection, _qh: &QueueHandle<Self>, _seat: wayland_client::protocol::wl_seat::WlSeat) {}
    fn remove_seat(&mut self, _conn: &wayland_client::Connection, _qh: &QueueHandle<Self>, _seat: wayland_client::protocol::wl_seat::WlSeat) {}
}

impl KeyboardHandler for LayerState {
    fn press_key(&mut self, _conn: &wayland_client::Connection, _qh: &QueueHandle<Self>, _keyboard: &wayland_client::protocol::wl_keyboard::WlKeyboard, _serial: u32, event: smithay_client_toolkit::seat::keyboard::KeyEvent) {
        if event.keysym == Keysym::Escape {
            self.close = true;
            return;
        }
        if event.keysym == Keysym::Return {
            self.select();
            return;
        }

        if event.keysym == Keysym::Down {
            self.selected = min((self.filter_results_cache.len() - 1) as u8, self.selected + 1);
        }
        if event.keysym == Keysym::Up {
            self.selected = if self.selected > 0 { self.selected - 1 } else { 0 };
            return;
        }
        if event.keysym == Keysym::Right {
            self.filter_input.advance_cursor();
        }
        if event.keysym == Keysym::Left {
            self.filter_input.reel_cursor();
        }

        if event.keysym == Keysym::BackSpace {
            if let Some(new_filter) = self.filter_input.pop_at_cursor() {
                self.filter = new_filter;
            }
        } else if let Some(character) = event.keysym.key_char() {
            let new_filter = self.filter_input.push_at_cursor(character);
            self.filter = new_filter;
        }

        // re-do results 
        self.filter_results = results::return_results(&self.filter);
        self.recreate_results_cache();
    }

    fn update_modifiers(&mut self, _conn: &wayland_client::Connection, _qh: &QueueHandle<Self>, _keyboard: &wayland_client::protocol::wl_keyboard::WlKeyboard, _serial: u32, _modifiers: smithay_client_toolkit::seat::keyboard::Modifiers, _layout: u32) {}
    fn enter(&mut self, _conn: &wayland_client::Connection, _qh: &QueueHandle<Self>, _keyboard: &wayland_client::protocol::wl_keyboard::WlKeyboard, _surface: &wayland_client::protocol::wl_surface::WlSurface, _serial: u32, _raw: &[u32], _keysyms: &[smithay_client_toolkit::seat::keyboard::Keysym]) {}
    fn leave(&mut self, _conn: &wayland_client::Connection, _qh: &QueueHandle<Self>, _keyboard: &wayland_client::protocol::wl_keyboard::WlKeyboard, _surface: &wayland_client::protocol::wl_surface::WlSurface, _serial: u32) {}
    fn release_key(&mut self, _conn: &wayland_client::Connection, _qh: &QueueHandle<Self>, _keyboard: &wayland_client::protocol::wl_keyboard::WlKeyboard, _serial: u32, _event: smithay_client_toolkit::seat::keyboard::KeyEvent) {}
}

impl ShmHandler for LayerState {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl ProvidesRegistryState for LayerState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

impl LayerState {
    pub fn draw(&mut self, qh: &QueueHandle<Self>) {
        let width = self.width;
        let height = self.height;
        let stride = self.width as i32 * 4;

        let (buffer, canvas) = self.pool.create_buffer(width as i32, height as i32, stride, wl_shm::Format::Argb8888).expect("Failed to create buffer on draw.");

        self.canvas.wipe(self.config.background_color);

        // Call your component draw calls here, in order you want them to display
        self.canvas.draw_box(0, 0, 1024, 48, self.config.foreground_color);
        self.filter_input.draw(&mut self.canvas);

        let selected_height = HEIGHT_PER_ELEMENT * self.selected as i32;
        if !self.filter_results_cache.is_empty() {
            self.canvas.draw_box(0, (49 + selected_height) as u32, 1024, HEIGHT_PER_ELEMENT as u32, self.config.selection_hover_color);
        } else {
            self.no_results_label.draw(&mut self.canvas);
        }
        self.canvas.draw_box(0, 49, 1024, 1, self.config.seperator_color);

        for x in &mut self.filter_results_cache {
            x.draw(&mut self.canvas);
        }


        // Push it to the surface
        self.canvas.fill_wayland_canvas(canvas);

        self.layer.wl_surface().damage_buffer(0, 0, width as i32, height as i32);
        self.layer.wl_surface().frame(qh, self.layer.wl_surface().clone());
        buffer.attach_to(self.layer.wl_surface()).expect("Failed to attach to buffer");
        self.layer.commit();
    }

    fn recreate_results_cache(&mut self) {
        let time: Instant = Instant::now();
        let mut transform = Vector2I::new(16, 49);
        let standard_size = Vector2I::new(1024, HEIGHT_PER_ELEMENT);

        // Math
        self.filter_results_cache = Vec::new();
        if let Some(math) = self.filter_results.math_result {
            let entry = Entrybox::new(EntryBoxValue::Math(math), transform, standard_size, self.config.font.clone());
            transform.set_y(transform.y() + HEIGHT_PER_ELEMENT);
            self.filter_results_cache.push(entry);
        }

        let mut count: u8 = 0;
        for desktop in &self.filter_results.desktop_results {
            let entry = Entrybox::new(EntryBoxValue::Desktop(desktop.to_owned()), transform, standard_size, self.config.font.clone());
            transform.set_y(transform.y() + HEIGHT_PER_ELEMENT);
            self.filter_results_cache.push(entry);
            count += 1;
            if count >= ELEMENT_LIMIT {
                break;
            }
        }

        println!("Time to recreate results element cache: {:?}", Instant::now() - time);
    }

    fn select(&mut self) {
        let selected = &self.filter_results_cache[self.selected as usize];
        selected.select();
        self.close = true;
    }
}

const HEIGHT_PER_ELEMENT: i32 = 30;
const ELEMENT_LIMIT: u8 = 50;

delegate_compositor!(LayerState);
delegate_output!(LayerState);
delegate_shm!(LayerState);
delegate_seat!(LayerState);
delegate_keyboard!(LayerState);
delegate_layer!(LayerState);
delegate_registry!(LayerState);

pub fn create_layer(config: SprintConfig) {
    let conn = Connection::connect_to_env().expect("Unable to connect to a compositor.");
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("Compositor does not support 'wl_compositor'");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("Compositor does not support 'zwlr_layer_shell_v1'");
    // software rendering because im too lazy to use wgpu
    let shm = Shm::bind(&globals, &qh).expect("Compositor does not support `wl_shm`");

    // create our surface and layer
    let width: u32 = 1024;
    let height: u32 = 512;
    let surface = compositor.create_surface(&qh);
    let layer = layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("sprint-layer"), None);
    layer.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
    layer.set_size(width, height);
    layer.commit();
    let pool = SlotPool::new((width * height * 4) as usize, &shm).expect("Failed to create pool");

    // state
    let font_source = SystemSource::new();
    let mut state = LayerState {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        shm,
        close: false,
        first_config: true,
        pool,
        layer,
        keyboard: None,
        canvas: RenderCanvas::new(width, height),
        width,
        height,

        filter: String::new(),
        filter_results: results::return_results(""),
        selected: 0,

        filter_input: InputBox::new("", "Search...", Vector2I::new(16, 8), Vector2I::new(996, 32), config.font.clone()),
        filter_results_cache: Vec::new(),
        no_results_label: TextLabel::new("¯\\_(._.)_/¯", config.font.clone(), 18.0, Vector2I::new(462, 240), Vector2I::new(100, 32)).expect("Unable to create no results label."),
        config
    };
    state.canvas.wipe(Color::new(25, 25, 25, 255));
    state.recreate_results_cache();

    // event loop
    loop {
        event_queue.blocking_dispatch(&mut state).unwrap();

        if state.close {
            break;
        }
    }
}
