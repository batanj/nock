use std::error::Error;

use smithay_client_toolkit::{
    compositor::CompositorState,
    shell::{
        WaylandSurface,
        wlr_layer::{Anchor, Layer, LayerShell, LayerSurface},
    },
};
use wayland_client::QueueHandle;

use crate::{
    SurfaceId, Widget,
    domain::{layout::Constraints, widget::HitTarget},
    gfx::egl::Egl,
    runtime::nock::Nock,
    theme::Theme,
};

pub struct Bar {
    pub surface_id: SurfaceId,
    pub layer_surface: Option<LayerSurface>,
    pub egl: Option<Egl>,
    pub width: u32,
    pub height: u32,
    pub root: Widget,
    pub hit_items: Vec<HitTarget>,
}

impl Bar {
    pub fn new(
        surface_id: SurfaceId,
        layer_surface: Option<LayerSurface>,
        egl: Option<Egl>,
        width: u32,
        height: u32,
        root: Widget,
        hit_items: Vec<HitTarget>,
    ) -> Self {
        Self {
            surface_id,
            layer_surface,
            egl,
            width,
            height,
            root,
            hit_items,
        }
    }

    pub fn create(
        layer_shell: &LayerShell,
        compositor: &CompositorState,
        qh: &QueueHandle<Nock>,
    ) -> Result<LayerSurface, Box<dyn Error>> {
        let surface = compositor.create_surface(qh);
        let layer = layer_shell.create_layer_surface(qh, surface, Layer::Top, Some("nock"), None);
        layer.set_anchor(Anchor::TOP | Anchor::LEFT | Anchor::RIGHT);
        layer.set_exclusive_zone(32);
        layer.set_size(0, 32);
        layer.wl_surface().commit();

        Ok(layer)
    }

    pub fn draw(&mut self, theme: &Theme, hover_id: &Option<String>) {
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
        egl.begin_frame(theme);
        for ht in items {
            if ht.id.is_some() && &ht.id == hover_id {
                egl.draw_label(&ht, &Theme::hover());
            } else {
                egl.draw_label(&ht, theme);
            }
        }
        egl.end_frame();
    }
}
