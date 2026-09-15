use std::ffi::c_void;

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_dispatch2, delegate_registry,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
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
    SurfaceId, Widget, domain::layout::Constraints, gfx::egl::Egl, runtime::app::App, theme::Theme,
};

pub struct Nock {
    pub app: App,
    pub clock_id: SurfaceId,
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub compositor_state: CompositorState,
    pub layer_shell: LayerShell,
    pub layer_surface: LayerSurface,
    pub width: u32,
    pub height: u32,
    pub exit: bool,
    pub egl: Option<Egl>,
    pub root: Widget,
    pub theme: Theme,
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
        self.theme = Theme::dark();
        egl.begin_frame(&self.theme);
        for (rect, text) in items {
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
        if configure.new_size.0 > 0 {
            self.width = configure.new_size.0
        }
        if configure.new_size.1 > 0 {
            self.height = configure.new_size.1
        }
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
        } else if let Some(egl) = &self.egl {
            egl.window
                .resize(self.width as i32, self.height as i32, 0, 0);
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

impl ProvidesRegistryState for Nock {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}

delegate_registry!(Nock);
delegate_dispatch2!(Nock);
