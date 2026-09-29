use std::{error::Error, ffi::c_void};

use smithay_client_toolkit::{
    compositor::CompositorState,
    shell::{
        WaylandSurface,
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerSurface, LayerSurfaceConfigure,
        },
    },
};
use wayland_client::{Proxy, QueueHandle};

use crate::{
    SurfaceId, Widget,
    domain::{layout::Constraints, widget::HitTarget},
    gfx::egl::Egl,
    runtime::{bar::BarAction, nock::Nock},
    theme::Theme,
};

pub struct Menu {
    pub layer_surface: Option<LayerSurface>,
    pub egl: Option<Egl>,
    pub width: u32,
    pub height: u32,
    pub id: SurfaceId,
    pub root: Widget,
    pub visible: bool,
    pub hit_items: Vec<HitTarget>,
}

impl Menu {
    pub fn new(
        layer_surface: Option<LayerSurface>,
        egl: Option<Egl>,
        width: u32,
        height: u32,
        id: SurfaceId,
    ) -> Self {
        Self {
            layer_surface,
            egl,
            width,
            height,
            id,
            root: Menu::menu(),
            visible: false,
            hit_items: vec![],
        }
    }
    pub fn create(
        layer_shell: &LayerShell,
        compositor: &CompositorState,
        qh: &QueueHandle<Nock>,
    ) -> Result<LayerSurface, Box<dyn Error>> {
        let menu_surface = compositor.create_surface(qh);
        let layer =
            layer_shell.create_layer_surface(qh, menu_surface, Layer::Top, Some("menu"), None);
        layer.set_anchor(Anchor::TOP | Anchor::LEFT);
        layer.set_exclusive_zone(0);
        layer.set_size(1, 1);

        layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);

        layer.wl_surface().commit();

        Ok(layer)
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
                .expect("menu egl bind"),
            );
        } else if let Some(ref mut egl) = self.egl
            && size_changed
            && new_w > 0
            && new_h > 0
        {
            egl.resize(self.width as i32, self.height as i32)
                .expect("menu egl resize");
        }
    }

    pub fn draw(&mut self, theme: &Theme) {
        if let Some(egl) = self.egl.as_mut() {
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
                egl.draw_label(&ht, theme);
            }
            egl.end_frame();
        }
    }

    pub fn hide(&mut self) {
        self.visible = false;
        if let Some(ref mut layer) = self.layer_surface {
            layer.set_size(1, 1);
            layer.wl_surface().commit();
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
        if let Some(ref mut layer) = self.layer_surface {
            layer.set_size(160, 200);
            layer.wl_surface().commit();
        }
    }

    pub fn action_at(&self, x: u32, y: u32) -> BarAction {
        let Some(t) = Widget::hit(&self.hit_items, x, y) else {
            return BarAction::None;
        };

        match t.on_click.as_deref() {
            Some("toggle_menu") => BarAction::ToggleMenu,
            _ => BarAction::None,
        }
    }

    pub fn menu() -> Widget {
        Widget::Column {
            children: vec![Widget::Label {
                text: "Menu".to_string(),
                id: None,
                on_click: None,
                command: None,
                pad: 8,
            }],
        }
    }
}
