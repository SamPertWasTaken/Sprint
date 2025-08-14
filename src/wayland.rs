use std::{cmp::min, num::NonZeroU32, time::Instant};

use pathfinder_geometry::vector::Vector2I;
use smithay_client_toolkit::{compositor::{CompositorHandler, CompositorState}, delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, delegate_registry, delegate_seat, delegate_shm, output::{OutputHandler, OutputState}, registry::{ProvidesRegistryState, RegistryState}, registry_handlers, seat::{keyboard::{KeyboardHandler, Keysym, RepeatInfo}, Capability, SeatHandler, SeatState}, shell::{wlr_layer::{KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface}, WaylandSurface}, shm::{slot::SlotPool, Shm, ShmHandler}};
use wayland_client::{globals::registry_queue_init, protocol::{wl_keyboard::WlKeyboard, wl_shm}, Connection, QueueHandle};

use crate::{entry_box::{EntryBoxValue, Entrybox}, input_box::InputBox, render_canvas::{CanvasRenderable, Color, RenderCanvas}, results::SprintResults, sprint_config::SprintConfig, text_label::TextLabel};

// the key to repeat -> the time it was pressed/last repeated -> if it is already repeating or
// is waiting for delay
struct RepeatKeyInfo(Keysym, Instant, bool);

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
    repeat_key: Option<RepeatKeyInfo>,
    repeat_delay: Option<u32>,
    repeat_rate: Option<NonZeroU32>,

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
        if self.repeat_delay.is_some() {
            self.repeat_key = Some(RepeatKeyInfo(event.keysym, Instant::now(), false));
        }
        self.key_press_handle(event.keysym);
    }

    fn update_repeat_info(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _keyboard: &wayland_client::protocol::wl_keyboard::WlKeyboard, info: RepeatInfo) {
        match info {
            RepeatInfo::Repeat { rate, delay } => {
                self.repeat_rate = Some(rate);
                self.repeat_delay = Some(delay);
            },
            RepeatInfo::Disable => {
                self.repeat_rate = None;
                self.repeat_delay = None;
            },
        }
    }
    fn release_key(&mut self, _conn: &wayland_client::Connection, _qh: &QueueHandle<Self>, _keyboard: &wayland_client::protocol::wl_keyboard::WlKeyboard, _serial: u32, event: smithay_client_toolkit::seat::keyboard::KeyEvent) {
        if let Some(RepeatKeyInfo(key, _, _)) = self.repeat_key {
            if event.keysym == key {
                self.repeat_key = None;
            }
        }
    }

    fn update_modifiers(&mut self, _conn: &wayland_client::Connection, _qh: &QueueHandle<Self>, _keyboard: &wayland_client::protocol::wl_keyboard::WlKeyboard, _serial: u32, _modifiers: smithay_client_toolkit::seat::keyboard::Modifiers, _layout: u32) {}
    fn enter(&mut self, _conn: &wayland_client::Connection, _qh: &QueueHandle<Self>, _keyboard: &wayland_client::protocol::wl_keyboard::WlKeyboard, _surface: &wayland_client::protocol::wl_surface::WlSurface, _serial: u32, _raw: &[u32], _keysyms: &[smithay_client_toolkit::seat::keyboard::Keysym]) {}
    fn leave(&mut self, _conn: &wayland_client::Connection, _qh: &QueueHandle<Self>, _keyboard: &wayland_client::protocol::wl_keyboard::WlKeyboard, _surface: &wayland_client::protocol::wl_surface::WlSurface, _serial: u32) {}
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
        let width_int = i32::try_from(width).expect("selected height to i32 failed");
        let height_int = i32::try_from(height).expect("height to i32 failed");
        let stride = width_int * 4;

        let (buffer, canvas) = self.pool.create_buffer(width_int, height_int, stride, wl_shm::Format::Argb8888).expect("Failed to create buffer on draw.");

        self.canvas.wipe(self.config.background_color);

        // Call your component draw calls here, in order you want them to display
        self.canvas.draw_box(0, 0, 1024, 48, self.config.foreground_color);
        self.filter_input.draw(&mut self.canvas);

        let selected_height = HEIGHT_PER_ELEMENT * i32::from(self.selected);
        if self.filter_results_cache.is_empty() {
            self.no_results_label.draw(&mut self.canvas);
        } else {
            self.canvas.draw_box(0, 49 + u32::try_from(selected_height).expect("selected height to u32 failed"), 1024, HEIGHT_PER_ELEMENT as u32, self.config.selection_hover_color);
        }
        self.canvas.draw_box(0, 49, 1024, 1, self.config.seperator_color);

        for x in &mut self.filter_results_cache {
            x.draw(&mut self.canvas);
        }


        // Push it to the surface
        self.canvas.fill_wayland_canvas(canvas);

        self.layer.wl_surface().damage_buffer(0, 0, width_int, height_int);
        self.layer.wl_surface().frame(qh, self.layer.wl_surface().clone());
        buffer.attach_to(self.layer.wl_surface()).expect("Failed to attach to buffer");
        self.layer.commit();
    }

    fn recreate_results_cache(&mut self) {
        let time = Instant::now();
        let mut transform = Vector2I::new(16, 49);
        let standard_size = Vector2I::new(1024, HEIGHT_PER_ELEMENT);
        self.filter_results_cache = Vec::new();

        for result_type in &self.config.result_order {
            match result_type.to_lowercase().as_str() {
                "prefixes" => {
                    if let Some(prefix) = &self.filter_results.prefix_results {
                        let prefix_entry = Entrybox::new(EntryBoxValue::WebPrefix(prefix.0.to_string(), prefix.1.to_string(), prefix.2.to_string()), transform, standard_size, self.config.font.clone());
                        transform.set_y(transform.y() + HEIGHT_PER_ELEMENT);
                        self.filter_results_cache.push(prefix_entry);
                    }
                },
                "math" => {
                    if let Some(math) = self.filter_results.math_result {
                        let entry = Entrybox::new(EntryBoxValue::Math(math), transform, standard_size, self.config.font.clone());
                        transform.set_y(transform.y() + HEIGHT_PER_ELEMENT);
                        self.filter_results_cache.push(entry);
                    }
                },
                "desktop" => {
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
                },
                "search" => {
                    let web_entry = Entrybox::new(EntryBoxValue::WebSearch(self.filter_results.web_result.0.to_string(), self.filter_results.web_result.1.to_string()), transform, standard_size, self.config.font.clone());
                    transform.set_y(transform.y() + HEIGHT_PER_ELEMENT);
                    self.filter_results_cache.push(web_entry);
                },
                _ => println!("Error: Unknown result type {result_type}")
            }
        }

        println!("Time to recreate results element cache: {:?}", time.elapsed());
    }

    fn select(&mut self) {
        let selected = &self.filter_results_cache[self.selected as usize];
        selected.select();
        self.close = true;
    }

    fn key_press_handle(&mut self, keysym: Keysym) {
        match keysym {
            // Control characters
            Keysym::Escape => self.close = true,
            Keysym::Return => self.select(),
            Keysym::BackSpace => if let Some(new_filter) = self.filter_input.pop_at_cursor() { self.filter = new_filter }
            // Cursor movement
            Keysym::Down => self.selected = min(u8::try_from(self.filter_results_cache.len() - 1).expect("filter results cache length to u8 failed"), self.selected + 1),
            Keysym::Up => self.selected = if self.selected != 0 { self.selected - 1} else { 0 },
            Keysym::Right => self.filter_input.advance_cursor(),
            Keysym::Left => self.filter_input.reel_cursor(),
            Keysym::Home => self.filter_input.set_cursor_to_home(),
            Keysym::End => self.filter_input.set_cursor_to_end(),
            
            _ => {
                if let Some(character) = keysym.key_char() {
                    self.filter = self.filter_input.push_at_cursor(character);
                }
            }
        }
        // re-do results 
        self.filter_results.refresh_results(&self.filter, &self.config);
        self.recreate_results_cache();
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
        repeat_key: None,
        repeat_delay: None,
        repeat_rate: None,

        filter: String::new(),
        filter_results: SprintResults::new(),
        selected: 0,

        filter_input: InputBox::new("", "Search...", Vector2I::new(16, 8), Vector2I::new(996, 32), &config.font),
        filter_results_cache: Vec::new(),
        no_results_label: TextLabel::new("¯\\_(._.)_/¯", config.font.clone(), 18.0, Vector2I::new(462, 240), Vector2I::new(100, 32)),
        config
    };
    state.filter_results.refresh_results("", &state.config);
    state.canvas.wipe(Color::new(25, 25, 25, 255));
    state.recreate_results_cache();

    // event loop
    loop {
        // key repetition
        // this var holds the key to repeat, just gets around the double mut borrow problem
        let mut key_repeat = None;
        // if we're holding down a key...
        if let Some(RepeatKeyInfo(key, ref mut time, ref mut active)) = state.repeat_key {
            let repeat_delay = state.repeat_delay.expect("repeat delay is set to none");
            let repeat_rate = state.repeat_rate.expect("repeat rate is set to none");
            // is the delay past yet?
            if !*active && time.elapsed().as_millis() > repeat_delay.into() {
                // if so, press the key and setup the repeatition
                *active = true;
                *time = Instant::now();
                key_repeat = Some(key);
            }

            // the actual repeptition once its active
            if *active {
                let char_rate = 1000 / repeat_rate;
                if time.elapsed().as_millis() > char_rate.into() {
                    *time = Instant::now();
                    key_repeat = Some(key);
                }
            }
        }
        // press it if needed
        if let Some(repeat) = key_repeat {
            state.key_press_handle(repeat);
        }

        // now back to boring wayland handling
        event_queue.blocking_dispatch(&mut state).unwrap();

        if state.close {
            break;
        }
    }
}
