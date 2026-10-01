use std::fs;
use std::path::PathBuf;
use std::time::{Instant, SystemTime};
use std::{error::Error, ffi::c_void};

use smithay_client_toolkit::{
    compositor::CompositorState,
    shell::{
        WaylandSurface,
        wlr_layer::{Anchor, Layer, LayerShell, LayerSurface, LayerSurfaceConfigure},
    },
};
use wayland_client::{Proxy, QueueHandle};

use crate::Action::Paint;
use crate::runtime::app::App;
use crate::{
    SurfaceId, Widget,
    config::lua_cfg::load_root,
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
    pub config_file: Option<PathBuf>,
    pub config_mtime: Option<SystemTime>,
    pub hover_id: Option<String>,
    pub theme: Theme,
}

pub enum BarAction {
    ToggleTheme,
    ToggleClock,
    ToggleMenu,
    TogglePanel,
    PrevMonth,
    NextMonth,
    Spawn { command: String, id: Option<String> },
    DismissPanel,
    SelectDay(u32),
    None,
}

impl Bar {
    pub fn new(
        surface_id: SurfaceId,
        layer_surface: Option<LayerSurface>,
        egl: Option<Egl>,
        width: u32,
        height: u32,
        hit_items: Vec<HitTarget>,
        theme: Theme,
    ) -> Self {
        let (root, config_file, config_mtime) = Bar::load_root();
        Self {
            surface_id,
            layer_surface,
            egl,
            width,
            height,
            root,
            hit_items,
            config_file,
            config_mtime,
            hover_id: None,
            theme,
        }
    }

    fn load_root() -> (Widget, Option<PathBuf>, Option<SystemTime>) {
        let (root, meta) = load_root("bar");
        let root = match root {
            Some(widget) => widget,
            None => Bar::default_row(),
        };

        let (config_file, config_mtime) = match meta {
            Some((p, t)) => (Some(p), Some(t)),
            None => (None, None),
        };

        (root, config_file, config_mtime)
    }

    pub fn action_at(&self, x: u32, y: u32) -> BarAction {
        let Some(t) = Widget::hit(&self.hit_items, x, y) else {
            return BarAction::DismissPanel;
        };

        match t.on_click.as_deref() {
            Some("toggle_theme") => BarAction::ToggleTheme,
            Some("spawn") => {
                let Some(cmd) = t.command.clone() else {
                    eprintln!("nock: spawn missing command");
                    return BarAction::None;
                };

                BarAction::Spawn {
                    command: cmd,
                    id: t.id.clone(),
                }
            }
            Some("toggle_clock") => BarAction::ToggleClock,
            Some("toggle_panel") => BarAction::TogglePanel,
            Some("toggle_menu") => BarAction::ToggleMenu,
            _ => BarAction::None,
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

    pub fn on_motion(&mut self, x: u32, y: u32) -> bool {
        let new = Widget::hit(&self.hit_items, x, y).and_then(|t| t.id.clone());
        if new != self.hover_id {
            self.hover_id = new;
            return true;
        }
        false
    }

    pub fn paint(&mut self, app: &mut App) {
        app.mark_dirty(self.surface_id);
        for (id, action) in app.tick(Instant::now()) {
            if action == Paint && id == self.surface_id {
                self.draw();
            }
        }
    }

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
        for ht in items {
            if ht.id.is_some() && ht.id == self.hover_id {
                egl.draw_label(&ht, &Theme::hover());
            } else {
                egl.draw_label(&ht, &self.theme);
            }
        }
        egl.end_frame();
    }

    pub fn default_row() -> Widget {
        Widget::Row {
            children: vec![
                Widget::Label {
                    text: "nock".into(),
                    id: Some("brand".into()),
                    on_click: Some("toggle_theme".to_string()),
                    command: None,
                    pad: 8,
                },
                Widget::Spacer,
                Widget::Clock {
                    text: "".into(),
                    id: Some("clock".into()),
                    on_click: None,
                    command: None,
                    pad: 8,
                },
                Widget::Date {
                    text: "".into(),
                    id: Some("date".into()),
                    on_click: None,
                    command: None,
                    pad: 8,
                },
            ],
        }
    }

    pub fn reload_if_changed(&mut self) {
        let Some(path) = self.config_file.as_ref() else {
            return;
        };
        let Ok(modified) = fs::metadata(path).and_then(|m| m.modified()) else {
            return;
        };
        if self.config_mtime.is_some_and(|t| modified <= t) {
            return;
        }

        let (root, config_file, config_mtime) = Bar::load_root();

        self.root = root;
        self.config_mtime = config_mtime;
        self.config_file = config_file;
    }
}
