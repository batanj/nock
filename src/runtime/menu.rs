use std::{
    error::Error,
    ffi::c_void,
    fs,
    path::PathBuf,
    time::{Instant, SystemTime},
};

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
    Action::Paint,
    SurfaceId, Widget,
    config::lua_cfg::load_root,
    domain::{layout::Constraints, widget::HitTarget},
    gfx::egl::Egl,
    runtime::{app::App, bar::BarAction, nock::Nock},
    theme::Theme,
};

pub struct Menu {
    pub layer_surface: Option<LayerSurface>,
    pub egl: Option<Egl>,
    pub width: u32,
    pub height: u32,
    pub surface_id: SurfaceId,
    pub root: Widget,
    pub visible: bool,
    pub hit_items: Vec<HitTarget>,
    pub config_file: Option<PathBuf>,
    pub config_mtime: Option<SystemTime>,
    pub hover_id: Option<String>,
}

impl Menu {
    pub fn new(
        layer_surface: Option<LayerSurface>,
        egl: Option<Egl>,
        width: u32,
        height: u32,
        surface_id: SurfaceId,
    ) -> Self {
        let (root, config_file, config_mtime) = Menu::load_root();

        Self {
            layer_surface,
            egl,
            width,
            height,
            surface_id,
            root,
            visible: false,
            hit_items: vec![],
            config_file,
            config_mtime,
            hover_id: None,
        }
    }

    fn load_root() -> (Widget, Option<PathBuf>, Option<SystemTime>) {
        let (root, meta) = load_root("menu");
        let root = match root {
            Some(widget) => widget,
            None => Menu::default_column(),
        };

        let (config_file, config_mtime) = match meta {
            Some((p, t)) => (Some(p), Some(t)),
            None => (None, None),
        };

        (root, config_file, config_mtime)
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

    pub fn on_motion(&mut self, x: u32, y: u32) -> bool {
        let new = Widget::hit(&self.hit_items, x, y).and_then(|t| t.id.clone());
        if new != self.hover_id {
            self.hover_id = new;
            return true;
        }
        false
    }

    pub fn paint(&mut self, app: &mut App, theme: &Theme) {
        app.mark_dirty(self.surface_id);
        for (id, action) in app.tick(Instant::now()) {
            if action == Paint && id == self.surface_id {
                self.draw(theme);
            }
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
                if ht.id.is_some() && ht.id == self.hover_id {
                    egl.draw_label(&ht, &Theme::hover());
                } else {
                    egl.draw_label(&ht, theme);
                }
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
            Some("toggle_theme") => BarAction::ToggleTheme,
            Some("toggle_menu") => BarAction::ToggleMenu,
            _ => BarAction::None,
        }
    }

    pub fn default_column() -> Widget {
        Widget::Column {
            children: vec![
                Widget::Label {
                    text: "Terminal".to_string(),
                    id: Some("kitty".to_string()),
                    on_click: Some("spawn".to_string()),
                    command: Some("kitty".to_string()),
                    pad: 8,
                },
                Widget::Label {
                    text: "Theme".to_string(),
                    id: Some("theme".to_string()),
                    on_click: Some("toggle_theme".to_string()),
                    command: None,
                    pad: 8,
                },
                Widget::Label {
                    text: "Close".to_string(),
                    id: Some("close".to_string()),
                    on_click: Some("toggle_menu".to_string()),
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

        let (root, config_file, config_mtime) = Menu::load_root();

        self.root = root;
        self.config_mtime = config_mtime;
        self.config_file = config_file;
    }
}
