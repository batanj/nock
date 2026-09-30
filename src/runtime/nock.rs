use std::{ffi::c_void, process::Command, time::Instant};

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_dispatch2, delegate_registry,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        Capability, SeatHandler, SeatState,
        keyboard::{KeyboardHandler, Keysym},
        pointer::{AxisScroll, PointerEventKind, PointerHandler},
    },
    shell::{
        WaylandSurface,
        wlr_layer::{LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
    },
};
use wayland_client::{
    Connection, QueueHandle,
    protocol::{wl_output, wl_surface},
};

use crate::runtime::{bar::Bar, menu::Menu, panel::Panel};
use crate::{
    SurfaceId, Widget,
    format::{format_now, volume_text},
    runtime::app::App,
    theme::Theme,
};

use super::bar::BarAction;

pub struct Nock {
    pub app: App,
    pub clock_id: SurfaceId,
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub seat_state: SeatState,
    pub compositor_state: CompositorState,
    pub layer_shell: LayerShell,
    pub display_ptr: *mut c_void,
    pub exit: bool,
    pub bar: Bar,
    pub panel: Option<Panel>,
    pub is_dark: bool,
    pub hour12: bool,
    pub last_net: Instant,
    pub menu: Option<Menu>,
}

impl Nock {
    fn on_axis(&mut self, _x: u32, _y: u32, vertical: &AxisScroll) {
        if vertical.absolute != 0.0 {
            let delta = if vertical.absolute > 0.0 {
                "5%+"
            } else {
                "5%-"
            };

            let _ = Command::new("wpctl")
                .args(["set-volume", "-l", "1.0", "@DEFAULT_AUDIO_SINK@", delta])
                .status();

            Widget::set_volume(&mut self.bar.root, &volume_text());
            self.bar.paint(&mut self.app);
        }
    }

    fn apply_action(&mut self, bar_action: BarAction) {
        match bar_action {
            BarAction::DismissPanel => {
                if let Some(panel) = self.panel.as_mut()
                    && panel.visible
                {
                    panel.hide();
                    panel.draw();
                }
                if let Some(menu) = self.menu.as_mut()
                    && menu.visible
                {
                    menu.hide();
                    menu.draw();
                }
            }
            BarAction::Spawn { command, id } => {
                let mut bits = command.split_whitespace();
                let Some(prog) = bits.next() else { return };

                if let Err(e) = std::process::Command::new(prog).args(bits).spawn() {
                    eprintln!("nock: spawn {command}: {e}");
                }

                if let Some(menu) = self.menu.as_mut()
                    && menu.visible
                {
                    menu.hide();
                    menu.draw();
                }

                if id.as_deref() == Some("volume") {
                    Widget::set_volume(&mut self.bar.root, &volume_text());
                    self.bar.paint(&mut self.app);
                }
            }
            BarAction::ToggleClock => {
                self.hour12 = !self.hour12;
                Widget::set_clock(&mut self.bar.root, &format_now(self.hour12));
                self.bar.paint(&mut self.app);
            }
            BarAction::ToggleMenu => {
                if let Some(ref mut menu) = self.menu {
                    match menu.visible {
                        true => menu.hide(),
                        false => menu.show(),
                    }
                    menu.draw();
                }
            }
            BarAction::TogglePanel => {
                if let Some(ref mut panel) = self.panel {
                    match panel.visible {
                        true => panel.hide(),
                        false => panel.show(),
                    }
                    panel.draw();
                }
            }
            BarAction::NextMonth => {
                if let Some(p) = self.panel.as_mut() {
                    p.shift_month(1);
                }
            }
            BarAction::PrevMonth => {
                if let Some(p) = self.panel.as_mut() {
                    p.shift_month(-1);
                }
            }
            BarAction::ToggleTheme => {
                if self.is_dark {
                    let theme = Theme::light();
                    self.bar.theme = theme.clone();
                    if let Some(m) = self.menu.as_mut() {
                        m.theme = theme.clone();
                        if m.visible {
                            m.draw();
                        }
                    }
                    if let Some(p) = self.panel.as_mut() {
                        p.theme = theme;
                        if p.visible {
                            p.draw();
                        }
                    }
                    self.is_dark = false;
                } else {
                    let theme = Theme::dark();
                    self.bar.theme = theme.clone();
                    if let Some(m) = self.menu.as_mut() {
                        m.theme = theme.clone();
                        if m.visible {
                            m.draw();
                        }
                    }
                    if let Some(p) = self.panel.as_mut() {
                        p.theme = theme.clone();
                        if p.visible {
                            p.draw();
                        }
                    }
                    self.is_dark = true;
                }
                self.bar.paint(&mut self.app);
            }
            BarAction::None => {}
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
        if Some(layer) == self.bar.layer_surface.as_ref() {
            self.exit = true;
        } else if self.panel.as_ref().and_then(|p| p.layer_surface.as_ref()) == Some(layer)
            && let Some(p) = self.panel.as_mut()
        {
            p.layer_surface = None;
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
        if Some(layer) == self.bar.layer_surface.as_ref() {
            self.bar.configure(configure, self.display_ptr, layer);
            self.bar.paint(&mut self.app);
        } else if let Some(p) = self.panel.as_mut()
            && Some(layer) == p.layer_surface.as_ref()
        {
            p.configure(configure, self.display_ptr, layer);
            p.draw();
            self.app.mark_dirty(p.surface_id);
        } else if let Some(m) = self.menu.as_mut()
            && Some(layer) == m.layer_surface.as_ref()
        {
            m.configure(configure, self.display_ptr, layer);
            m.draw();
            self.app.mark_dirty(m.surface_id);
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
        if let Some(menu) = &mut self.menu
            && menu.visible
            && event.keysym == Keysym::Escape
        {
            menu.hide();
            menu.draw();
            return;
        }

        if let Some(panel) = &mut self.panel
            && panel.visible
        {
            match event.keysym {
                Keysym::Escape => {
                    panel.hide();
                    panel.draw();
                }
                Keysym::Left => {
                    panel.shift_month(-1);
                }
                Keysym::Right => {
                    panel.shift_month(1);
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
                    if let Some(panel) = &mut self.panel.as_mut()
                        && panel.visible
                        && let Some(layer) = &panel.layer_surface
                        && &event.surface == layer.wl_surface()
                        && panel.on_motion(x, y)
                    {
                        panel.paint(&mut self.app);
                        continue;
                    }

                    if let Some(layer) = &self.bar.layer_surface
                        && layer.wl_surface() == &event.surface
                        && self.bar.on_motion(x, y)
                    {
                        self.bar.paint(&mut self.app);
                    }

                    if let Some(ref mut menu) = self.menu
                        && let Some(ref mut layer) = menu.layer_surface
                        && layer.wl_surface() == &event.surface
                        && menu.on_motion(x, y)
                    {
                        menu.paint(&mut self.app);
                    }
                }
                PointerEventKind::Leave { serial: _ } => {
                    if let Some(panel) = &mut self.panel
                        && panel.visible
                        && let Some(layer) = &panel.layer_surface
                        && &event.surface == layer.wl_surface()
                        && panel.hover_id.take().is_some()
                    {
                        panel.paint(&mut self.app);
                        continue;
                    }

                    if self.bar.hover_id.take().is_some() {
                        self.bar.paint(&mut self.app);
                    }

                    if let Some(ref mut menu) = self.menu
                        && menu.hover_id.take().is_some()
                    {
                        menu.paint(&mut self.app);
                    }
                }
                PointerEventKind::Axis {
                    time: _,
                    horizontal: _,
                    vertical,
                    source: _,
                } => {
                    if let Some(layer_surface) = &self.bar.layer_surface
                        && &event.surface != layer_surface.wl_surface()
                    {
                        continue;
                    }
                    if Widget::hit(&self.bar.hit_items, x, y).and_then(|t| t.id.as_deref())
                        != Some("volume")
                    {
                        continue;
                    }

                    self.on_axis(x, y, vertical);
                }

                PointerEventKind::Press {
                    time: _,
                    button: _,
                    serial: _,
                } => {
                    if let Some(panel) = &mut self.panel
                        && panel.visible
                        && let Some(layer) = &panel.layer_surface
                        && &event.surface == layer.wl_surface()
                    {
                        let action = panel.action_at(x, y);
                        self.apply_action(action);
                    } else if let Some(menu) = &mut self.menu
                        && menu.visible
                        && let Some(layer) = &menu.layer_surface
                        && &event.surface == layer.wl_surface()
                    {
                        let action = menu.action_at(x, y);
                        self.apply_action(action);
                    } else if let Some(layer_surface) = &mut self.bar.layer_surface
                        && &event.surface == layer_surface.wl_surface()
                    {
                        let action = self.bar.action_at(x, y);
                        self.apply_action(action);
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
