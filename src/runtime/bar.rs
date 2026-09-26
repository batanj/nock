use std::{error::Error, ffi::c_void};

use smithay_client_toolkit::{
    compositor::CompositorState,
    shell::{
        WaylandSurface,
        wlr_layer::{Anchor, Layer, LayerShell, LayerSurface, LayerSurfaceConfigure},
    },
};
use wayland_client::{Proxy, QueueHandle};

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

    pub fn configure(
        &mut self,
        configure: LayerSurfaceConfigure,
        display_ptr: *mut c_void,
        layer_surface: &LayerSurface,
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

        if self.egl.is_none() && self.width > 0 && self.height > 0 {
            self.egl = Some(
                Egl::bind(
                    display_ptr,
                    layer_surface.wl_surface().id(),
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
