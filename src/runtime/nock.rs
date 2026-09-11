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
    shm::{Shm, ShmHandler},
};
use wayland_client::{
    Connection, Proxy, QueueHandle,
    protocol::{wl_output, wl_surface},
};

use crate::{SurfaceId, gfx::egl::Egl, runtime::app::App};

pub struct Nock {
    pub app: App,
    pub clock_id: SurfaceId,
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub shm: Shm,
    pub compositor_state: CompositorState,
    pub layer_shell: LayerShell,
    pub layer_surface: LayerSurface,
    pub width: u32,
    pub height: u32,
    pub exit: bool,
    pub red: bool,
    pub egl: Option<Egl>,
}

impl Nock {
    pub fn draw(&mut self) {
        let Some(egl) = &self.egl else { return };
        if self.red {
            egl.clear_rgb(1.0, 0.0, 0.0)
        } else {
            egl.clear_rgb(0.0, 0.0, 1.0)
        };
        egl.swap().expect("egl swap");
        println!("egl swap {}x{}", self.width, self.height);
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
        serial: u32,
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
            println!("{id:?} -> {action:?}");
            if action == crate::Action::Paint && id == self.clock_id {
                self.draw();
            }
        }

        // self.draw(qh);
        println!("draw {}x{}", self.width, self.height);
        println!("configure serial={serial} size={:?}", configure.new_size);
    }
}

impl ShmHandler for Nock {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
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
