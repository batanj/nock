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
    SurfaceId, Widget, domain::layout::Constraints, gfx::egl::Egl, runtime::nock::Nock,
    theme::Theme,
};

pub struct Panel {
    pub layer: Option<LayerSurface>,
    pub egl: Option<Egl>,
    pub width: u32,
    pub height: u32,
    pub id: SurfaceId,
    pub root: Widget,
    pub visible: bool,
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
            visible: false,
            root: Widget::Column {
                children: vec![
                    Widget::Label {
                        text: "Mon".to_string(),
                        id: None,
                        on_click: None,
                        command: None,
                        pad: 8,
                    },
                    Widget::Label {
                        text: "Tue".to_string(),
                        id: None,
                        on_click: None,
                        command: None,
                        pad: 8,
                    },
                    Widget::Label {
                        text: "Wed".to_string(),
                        id: None,
                        on_click: None,
                        command: None,
                        pad: 8,
                    },
                    Widget::Label {
                        text: "Thu".to_string(),
                        id: None,
                        on_click: None,
                        command: None,
                        pad: 8,
                    },
                    Widget::Label {
                        text: "Fri".to_string(),
                        id: None,
                        on_click: None,
                        command: None,
                        pad: 8,
                    },
                    Widget::Label {
                        text: "Sat".to_string(),
                        id: None,
                        on_click: None,
                        command: None,
                        pad: 8,
                    },
                    Widget::Label {
                        text: "Sun".to_string(),
                        id: None,
                        on_click: None,
                        command: None,
                        pad: 8,
                    },
                ],
            },
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
        layer.set_size(1, 1);
        layer.wl_surface().commit();

        Ok(layer)
    }

    pub fn hide(panel: &mut Panel) {
        println!("panel hiding?");
        panel.visible = false;
        if let Some(ref mut layer) = panel.layer {
            layer.set_size(1, 1);
        }
    }

    pub fn show(panel: &mut Panel) {
        println!("panel showing?");
        panel.visible = true;
        if let Some(ref mut layer) = panel.layer {
            layer.set_size(120, 240);
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
            egl.begin_frame(theme);
            for ht in items {
                egl.draw_label(&ht, theme);
            }
            egl.end_frame();
        }
    }
}
