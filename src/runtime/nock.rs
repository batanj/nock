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
    shm::{Shm, ShmHandler, slot::SlotPool},
};
use wayland_client::{
    Connection, QueueHandle,
    protocol::{wl_output, wl_shm, wl_surface},
};

use crate::{SurfaceId, runtime::app::App};

pub struct Nock {
    pub app: App,
    pub clock_id: SurfaceId,
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub shm: Shm,
    pub compositor_state: CompositorState,
    pub layer_shell: LayerShell,
    pub layer_surface: LayerSurface,
    pub pool: SlotPool,
    pub width: u32,
    pub height: u32,
    pub exit: bool,
}

impl Nock {
    fn draw(&mut self, qh: &QueueHandle<Self>) {
        let width = self.width as i32;
        let height = self.height as i32;
        let stride = width * 4;

        let (buffer, canvas) = self
            .pool
            .create_buffer(width, height, stride, wl_shm::Format::Argb8888)
            .expect("create buffer");

        for px in canvas.as_chunks_mut::<4>().0 {
            px.copy_from_slice(&[0x00, 0x00, 0xFF, 0xFF]);
        }

        self.layer_surface
            .wl_surface()
            .damage_buffer(0, 0, width, height);
        buffer
            .attach_to(self.layer_surface.wl_surface())
            .expect("attach");
        self.layer_surface.commit();
    }
}

impl CompositorHandler for Nock {
    fn scale_factor_changed(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        new_factor: i32,
    ) {
    }

    fn frame(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        time: u32,
    ) {
    }

    fn surface_enter(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        output: &wl_output::WlOutput,
    ) {
    }

    fn transform_changed(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        new_transform: wayland_client::protocol::wl_output::Transform,
    ) {
    }
}

impl LayerShellHandler for Nock {
    fn closed(&mut self, conn: &Connection, qh: &QueueHandle<Self>, layer: &LayerSurface) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        serial: u32,
    ) {
        if configure.new_size.0 > 0 {
            self.width = configure.new_size.0
        }
        if configure.new_size.1 > 0 {
            self.height = configure.new_size.1
        }

        self.app.mark_dirty(self.clock_id);

        let now = std::time::Instant::now();
        for (id, action) in self.app.tick(now) {
            println!("{id:?} -> {action:?}");
            if action == crate::Action::Paint && id == self.clock_id {
                self.draw(qh);
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
        conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
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
