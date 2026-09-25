use std::{
    ffi::c_void,
    path::PathBuf,
    process::Command,
    time::{Instant, SystemTime},
};

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_dispatch2, delegate_registry,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        Capability, SeatHandler, SeatState,
        keyboard::{KeyboardHandler, Keysym},
        pointer::{PointerEventKind, PointerHandler},
    },
    shell::{
        WaylandSurface,
        wlr_layer::{LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
    },
};
use wayland_client::{
    Connection, Proxy, QueueHandle,
    protocol::{wl_output, wl_surface},
};

use crate::runtime::panel::Panel;
use crate::{
    Action::Paint,
    SurfaceId, Widget,
    domain::{layout::Constraints, widget::HitTarget},
    format::{format_now, volume_text},
    gfx::egl::Egl,
    runtime::app::App,
    theme::Theme,
};

pub struct Nock {
    pub app: App,
    pub clock_id: SurfaceId,
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub seat_state: SeatState,
    pub compositor_state: CompositorState,
    pub layer_shell: LayerShell,
    pub layer_surface: LayerSurface,
    pub display_ptr: *mut c_void,
    pub width: u32,
    pub height: u32,
    pub exit: bool,
    pub egl: Option<Egl>,
    pub panel: Option<Panel>,
    pub root: Widget,
    pub config_file: Option<PathBuf>,
    pub config_mtime: Option<SystemTime>,
    pub theme: Theme,
    pub hit_items: Vec<HitTarget>,
    pub is_dark: bool,
    pub hover_id: Option<String>,
    pub hour12: bool,
    pub last_net: Instant,
}

impl Nock {
    pub fn draw(&mut self) {
        let Some(egl) = &mut self.egl else { return };
        let items = Widget::layout(
            &self.root,
            &Constraints {
                max_width: self.width,
                max_height: self.height,
            },
            egl,
        )
        .1;
        self.hit_items = items.clone();
        egl.begin_frame(&self.theme);
        for ht in items {
            if ht.id.is_some() && ht.id == self.hover_id {
                egl.draw_label(&ht, &Theme::hover());
            } else {
                egl.draw_label(&ht, &self.theme);
            }
        }
        egl.end_frame();
    }
    fn request_paint(&mut self) {
        self.app.mark_dirty(self.clock_id);
        for (id, action) in self.app.tick(Instant::now()) {
            if action == Paint && id == self.clock_id {
                self.draw();
            }
        }
    }
}

impl CompositorHandler for Nock {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wayland_client::protocol::wl_output::Transform,
    ) {
    }
}

impl LayerShellHandler for Nock {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, layer: &LayerSurface) {
        if layer == &self.layer_surface {
            self.exit = true;
        } else if self.panel.as_ref().and_then(|p| p.layer.as_ref()) == Some(layer)
            && let Some(p) = self.panel.as_mut()
        {
            p.layer = None;
            p.egl = None;
        }
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        if layer == &self.layer_surface {
            let mut new_w = self.width;
            let mut new_h = self.height;

            if configure.new_size.0 > 0 {
                new_w = configure.new_size.0;
            }
            if configure.new_size.1 > 0 {
                new_h = configure.new_size.1;
            }

            let size_changed = new_w != self.width || new_h != self.height;

            self.width = new_w;
            self.height = new_h;

            if self.egl.is_none() && self.width > 0 && self.height > 0 {
                self.egl = Some(
                    Egl::bind(
                        self.display_ptr,
                        self.layer_surface.wl_surface().id(),
                        self.width as i32,
                        self.height as i32,
                    )
                    .expect("egl bind"),
                );
            } else if let Some(ref mut egl) = self.egl
                && size_changed
                && new_w > 0
                && new_h > 0
            {
                egl.resize(self.width as i32, self.height as i32)
                    .expect("egl resize");
            }

            self.request_paint();
        } else if let Some(p) = self.panel.as_mut()
            && Some(layer) == p.layer.as_ref()
        {
            let mut new_w = p.width;
            let mut new_h = p.height;

            if configure.new_size.0 > 0 {
                new_w = configure.new_size.0;
            }
            if configure.new_size.1 > 0 {
                new_h = configure.new_size.1;
            }

            let size_changed = new_w != p.width || new_h != p.height;

            p.width = new_w;
            p.height = new_h;

            if p.egl.is_none() {
                p.egl = Some(
                    Egl::bind(
                        self.display_ptr,
                        layer.wl_surface().id(),
                        p.width as i32,
                        p.height as i32,
                    )
                    .expect("panel egl bind"),
                );
            } else if size_changed {
                p.egl
                    .as_mut()
                    .map(|e| Some(e.resize(new_w as i32, new_h as i32)));
            }

            p.draw(&self.theme);
            self.app.mark_dirty(p.id);
        }
    }
}

impl OutputHandler for Nock {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl KeyboardHandler for Nock {
    fn enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wayland_client::protocol::wl_keyboard::WlKeyboard,
        _surface: &wl_surface::WlSurface,
        _serial: u32,
        _raw: &[u32],
        _keysyms: &[Keysym],
    ) {
    }

    fn leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wayland_client::protocol::wl_keyboard::WlKeyboard,
        _surface: &wl_surface::WlSurface,
        _serial: u32,
    ) {
    }

    fn press_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wayland_client::protocol::wl_keyboard::WlKeyboard,
        _serial: u32,
        event: smithay_client_toolkit::seat::keyboard::KeyEvent,
    ) {
        if let Some(panel) = &mut self.panel
            && panel.visible
        {
            match event.keysym {
                Keysym::Escape => {
                    Panel::hide(panel);
                    panel.draw(&self.theme);
                }
                Keysym::Left => {
                    panel.shift_month(-1, &self.theme);
                }
                Keysym::Right => {
                    panel.shift_month(1, &self.theme);
                }
                _ => {}
            }
        }
    }

    fn repeat_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wayland_client::protocol::wl_keyboard::WlKeyboard,
        _serial: u32,
        _event: smithay_client_toolkit::seat::keyboard::KeyEvent,
    ) {
    }

    fn release_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wayland_client::protocol::wl_keyboard::WlKeyboard,
        _serial: u32,
        _event: smithay_client_toolkit::seat::keyboard::KeyEvent,
    ) {
    }

    fn update_modifiers(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wayland_client::protocol::wl_keyboard::WlKeyboard,
        _serial: u32,
        _modifiers: smithay_client_toolkit::seat::keyboard::Modifiers,
        _raw_modifiers: smithay_client_toolkit::seat::keyboard::RawModifiers,
        _layout: u32,
    ) {
    }
}

impl PointerHandler for Nock {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wayland_client::protocol::wl_pointer::WlPointer,
        events: &[smithay_client_toolkit::seat::pointer::PointerEvent],
    ) {
        for event in events {
            let x = event.position.0 as u32;
            let y = event.position.1 as u32;

            match &event.kind {
                PointerEventKind::Motion { time: _ } => {
                    if let Some(panel) = &mut self.panel
                        && panel.visible
                        && let Some(layer) = &panel.layer
                        && &event.surface == layer.wl_surface()
                    {
                        continue;
                    }

                    let new = Widget::hit(&self.hit_items, x, y).and_then(|t| t.id.clone());
                    if new != self.hover_id {
                        self.hover_id = new;
                        self.request_paint();
                    }
                }
                PointerEventKind::Leave { serial: _ } => {
                    if let Some(panel) = &mut self.panel
                        && panel.visible
                        && let Some(layer) = &panel.layer
                        && &event.surface == layer.wl_surface()
                    {
                        continue;
                    }

                    if self.hover_id.take().is_some() {
                        self.request_paint();
                    }
                }
                PointerEventKind::Axis {
                    time: _,
                    horizontal: _,
                    vertical,
                    source: _,
                } => {
                    if event.surface != *self.layer_surface.wl_surface() {
                        continue;
                    }
                    if Widget::hit(&self.hit_items, x, y).and_then(|t| t.id.as_deref())
                        != Some("volume")
                    {
                        continue;
                    }

                    if vertical.absolute != 0.0 {
                        let delta = if vertical.absolute > 0.0 {
                            "5%+"
                        } else {
                            "5%-"
                        };

                        let _ = Command::new("wpctl")
                            .args(["set-volume", "-l", "1.0", "@DEFAULT_AUDIO_SINK@", delta])
                            .status();

                        Widget::set_volume(&mut self.root, &volume_text());
                        self.request_paint();
                    }
                }

                PointerEventKind::Press {
                    time: _,
                    button: _,
                    serial: _,
                } => {
                    if let Some(panel) = &mut self.panel
                        && panel.visible
                        && let Some(layer) = &panel.layer
                        && &event.surface == layer.wl_surface()
                    {
                        if let Some(t) = Widget::hit(&panel.hit_items, x, y) {
                            match t.id.as_deref() {
                                Some("prev") => {
                                    panel.shift_month(-1, &self.theme);
                                }
                                Some("next") => {
                                    panel.shift_month(1, &self.theme);
                                }
                                _ => {}
                            }
                        }
                        continue;
                    } else if event.surface != *self.layer_surface.wl_surface() {
                        continue;
                    } else if let Some(t) = Widget::hit(&self.hit_items, x, y) {
                        match t.on_click.as_deref() {
                            Some("toggle_theme") => {
                                if self.is_dark {
                                    self.theme = Theme::light();
                                    self.is_dark = false;
                                } else {
                                    self.theme = Theme::dark();
                                    self.is_dark = true;
                                }
                                self.request_paint();
                            }
                            Some("spawn") => {
                                let Some(cmd) = t.command.as_deref() else {
                                    eprintln!("nock: spawn missing command");
                                    continue;
                                };

                                let mut bits = cmd.split_whitespace();
                                let Some(prog) = bits.next() else {
                                    eprintln!("nock: spawn missing command");
                                    continue;
                                };

                                if let Err(e) = std::process::Command::new(prog).args(bits).spawn()
                                {
                                    eprintln!("nock: spawn {cmd}: {e}");
                                }

                                if let Some(id) = &t.id
                                    && id == "volume"
                                {
                                    Widget::set_volume(&mut self.root, &volume_text());
                                    self.request_paint();
                                }
                            }
                            Some("toggle_clock") => {
                                self.hour12 = !self.hour12;
                                Widget::set_clock(&mut self.root, &format_now(self.hour12));
                                self.request_paint();
                            }
                            Some("toggle_panel") => {
                                if let Some(ref mut panel) = self.panel
                                    && panel.visible
                                {
                                    panel.hide();
                                    panel.draw(&self.theme);
                                    continue;
                                }

                                if let Some(ref mut panel) = self.panel {
                                    panel.show();
                                    panel.draw(&self.theme);
                                }
                            }
                            Some(other) => eprintln!("unknown on_click {other}"),
                            None => {}
                        }
                    } else if let Some(panel) = self.panel.as_mut()
                        && panel.visible
                    {
                        Panel::hide(panel);
                        panel.draw(&self.theme);
                    }
                }
                _ => {}
            }
        }
    }
}
impl SeatHandler for Nock {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }
    fn new_seat(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: wayland_client::protocol::wl_seat::WlSeat,
    ) {
    }
    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wayland_client::protocol::wl_seat::WlSeat,
        capability: smithay_client_toolkit::seat::Capability,
    ) {
        if capability == Capability::Pointer {
            let _ = self.seat_state.get_pointer(qh, &seat);
        }
        if capability == Capability::Keyboard {
            let _ = self.seat_state.get_keyboard(qh, &seat, None);
        }
    }
    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: wayland_client::protocol::wl_seat::WlSeat,
        _capability: smithay_client_toolkit::seat::Capability,
    ) {
    }
    fn remove_seat(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: wayland_client::protocol::wl_seat::WlSeat,
    ) {
    }
}

impl ProvidesRegistryState for Nock {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

delegate_registry!(Nock);
delegate_dispatch2!(Nock);
