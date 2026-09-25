use std::error::Error;

use chrono::{Datelike, NaiveDate};
use smithay_client_toolkit::{
    compositor::CompositorState,
    shell::{
        WaylandSurface,
        wlr_layer::{Anchor, KeyboardInteractivity, Layer, LayerShell, LayerSurface},
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

pub struct Panel {
    pub layer: Option<LayerSurface>,
    pub egl: Option<Egl>,
    pub width: u32,
    pub height: u32,
    pub id: SurfaceId,
    pub root: Widget,
    pub visible: bool,
    pub year: i32,
    pub month: u32,
    pub hit_items: Vec<HitTarget>,
}

impl Panel {
    pub fn new(
        layer: Option<LayerSurface>,
        egl: Option<Egl>,
        width: u32,
        height: u32,
        id: SurfaceId,
    ) -> Self {
        let cur = chrono::Local::now();
        let (year, month) = (cur.year(), cur.month());
        Self {
            layer,
            egl,
            width,
            height,
            id,
            visible: false,
            root: Panel::month_grid(year, month),
            year,
            month,
            hit_items: vec![],
        }
    }

    pub fn month_grid(year: i32, month: u32) -> Widget {
        let current_date = chrono::Local::now();
        let date = NaiveDate::from_ymd_opt(year, month, 1).unwrap();

        let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let (next_year, next_month) = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };
        let first_of_next = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap();
        let total_days = (first_of_next - first_day).num_days();
        let leading_spaces = first_day.weekday().num_days_from_sunday() as usize;

        let day_pad = 8;

        let days_header: Vec<Widget> = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"]
            .iter()
            .map(|d| Widget::Label {
                text: d.to_string(),
                id: None,
                on_click: None,
                command: None,
                pad: day_pad,
            })
            .collect();

        let mut widget_column = vec![
            Widget::Row {
                children: vec![
                    Widget::Label {
                        text: String::from("<"),
                        id: Some("prev".to_string()),
                        on_click: None,
                        command: None,
                        pad: day_pad,
                    },
                    Widget::Label {
                        text: date.format("%B %Y").to_string(),
                        id: None,
                        on_click: None,
                        command: None,
                        pad: day_pad,
                    },
                    Widget::Label {
                        text: String::from(">"),
                        id: Some("next".to_string()),
                        on_click: None,
                        command: None,
                        pad: day_pad,
                    },
                ],
            },
            Widget::Row {
                children: days_header,
            },
        ];

        let mut days = (0..(leading_spaces + total_days as usize))
            .map(|i| {
                let day = (i + 1).saturating_sub(leading_spaces);
                let text = match i {
                    i if i < leading_spaces => String::from("  "),
                    _ => format!("{:2}", day),
                };

                let id = if day as u32 == current_date.day()
                    && month == current_date.month()
                    && year == current_date.year()
                {
                    Some(String::from("highlight"))
                } else {
                    None
                };

                Widget::Label {
                    text,
                    id,
                    on_click: None,
                    command: None,
                    pad: day_pad,
                }
            })
            .collect::<Vec<Widget>>();

        while !days.is_empty() {
            let n = days.len().min(7);
            let children = days.drain(..n).collect();

            widget_column.push(Widget::Row { children });
        }

        Widget::Column {
            children: widget_column,
        }
    }

    pub fn shift_month(&mut self, delta: i32, theme: &Theme) {
        let total_months = self.year * 12 + (self.month as i32 - 1) + delta;

        self.year = total_months.div_euclid(12);
        self.month = (total_months.rem_euclid(12) + 1) as u32;

        self.root = Panel::month_grid(self.year, self.month);
        self.draw(theme);
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

        layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);

        layer.wl_surface().commit();

        Ok(layer)
    }

    pub fn on_panel_press(&mut self, x: u32, y: u32, theme: &Theme) {
        if let Some(t) = Widget::hit(&self.hit_items, x, y) {
            match t.id.as_deref() {
                Some("prev") => {
                    self.shift_month(-1, theme);
                }
                Some("next") => {
                    self.shift_month(1, theme);
                }
                _ => {}
            }
        }
    }

    pub fn hide(&mut self) {
        self.visible = false;
        if let Some(ref mut layer) = self.layer {
            layer.set_size(1, 1);
            layer.wl_surface().commit();
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
        if let Some(ref mut layer) = self.layer {
            layer.set_size(240, 200);
            layer.wl_surface().commit();
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
                if let Some(id) = &ht.id
                    && id == &String::from("highlight")
                {
                    egl.draw_label(&ht, &Theme::hover());
                } else {
                    egl.draw_label(&ht, theme);
                }
            }
            egl.end_frame();
        }
    }
}
