use std::error::Error;

use smithay_client_toolkit::{
    compositor::CompositorState,
    shell::{
        WaylandSurface,
        wlr_layer::{Anchor, Layer, LayerShell, LayerSurface},
    },
};
use wayland_client::QueueHandle;

use crate::{SurfaceId, gfx::egl::Egl, runtime::nock::Nock, theme::Theme};

pub struct Panel {
    pub layer: Option<LayerSurface>,
    pub egl: Option<Egl>,
    pub width: u32,
    pub height: u32,
    pub id: SurfaceId,
}

impl Panel {
    pub fn new(
        layer: Option<LayerSurface>,
        egl: Option<Egl>,
        width: u32,
        height: u32,
        id: SurfaceId,
    ) -> Self {
        Self {
            layer,
            egl,
            width,
            height,
            id,
        }
    }

    pub fn create(
        layer_shell: &LayerShell,
        compositor: &CompositorState,
        qh: &QueueHandle<Nock>,
    ) -> Result<LayerSurface, Box<dyn Error>> {
        let panel_surface = compositor.create_surface(qh);
        let layer =
            layer_shell.create_layer_surface(qh, panel_surface, Layer::Top, Some("panel"), None);
        layer.set_anchor(Anchor::TOP | Anchor::LEFT);
        layer.set_exclusive_zone(0);
        layer.set_size(120, 240);
        layer.wl_surface().commit();

        Ok(layer)
    }

    pub fn destroy(panel: &mut Panel) {
        panel.egl = None;
        panel.layer = None;
    }

    pub fn draw(&mut self, theme: &Theme) {
        if let Some(egl) = self.egl.as_mut() {
            egl.begin_frame(theme);

            egl.end_frame();
        }
    }
}
