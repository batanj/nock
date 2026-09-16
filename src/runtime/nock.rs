use std::{ffi::c_void, time::Instant};

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_dispatch2, delegate_registry,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        Capability, SeatHandler, SeatState,
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

use crate::{
    Action::Paint,
    SurfaceId, Widget,
    domain::layout::{Constraints, Rect},
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
    pub width: u32,
    pub height: u32,
    pub exit: bool,
    pub egl: Option<Egl>,
    pub root: Widget,
    pub theme: Theme,
    pub hit_items: Vec<(Rect, String, Option<String>)>,
    pub is_dark: bool,
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
        for (rect, text, _id) in items {
            egl.draw_label(&text, rect, &self.theme);
        }
        egl.end_frame();
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
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
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

        let display_ptr = conn.backend().display_ptr() as *mut c_void;

        if self.egl.is_none() && self.width > 0 && self.height > 0 {
            self.egl = Some(
                Egl::bind(
                    display_ptr,
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

        self.app.mark_dirty(self.clock_id);

        let now = std::time::Instant::now();
        for (id, action) in self.app.tick(now) {
            if action == crate::Action::Paint && id == self.clock_id {
                self.draw();
            }
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

impl PointerHandler for Nock {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wayland_client::protocol::wl_pointer::WlPointer,
        events: &[smithay_client_toolkit::seat::pointer::PointerEvent],
    ) {
        for event in events {
            if let PointerEventKind::Press {
                time: _,
                button: _,
                serial: _,
            } = &event.kind
            {
                if event.surface != *self.layer_surface.wl_surface() {
                    continue;
                }
                let x = event.position.0 as u32;
                let y = event.position.1 as u32;

                if let Some("brand") = Widget::hit(&self.hit_items, x, y) {
                    if self.is_dark {
                        self.theme = Theme::light();
                        self.is_dark = false;
                    } else {
                        self.theme = Theme::dark();
                        self.is_dark = true;
                    }
                    self.app.mark_dirty(self.clock_id);
                    for (id, action) in self.app.tick(Instant::now()) {
                        if action == Paint && id == self.clock_id {
                            self.draw();
                        }
                    }
                }
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
