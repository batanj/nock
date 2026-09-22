use std::error::Error;

use chrono::{Datelike, NaiveDate};
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
            root: Panel::date(),
        }
    }

    pub fn date() -> Widget {
        let current_date = chrono::Local::now();
        let year = current_date.year();
        let month = current_date.month();

        let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let (next_year, next_month) = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };
        let first_of_next = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap();
        let total_days = (first_of_next - first_day).num_days();
        let leading_spaces = first_day.weekday().num_days_from_sunday() as usize;

        let day_pad = 4;

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
            Widget::Label {
                text: current_date.format("%B %Y").to_string(),
                id: None,
                on_click: None,
                command: None,
                pad: 8,
            },
            Widget::Row {
                children: days_header,
            },
        ];

        let mut days = (0..(leading_spaces + total_days as usize))
            .map(|i| {
                let text = match i {
                    i if i < leading_spaces => String::from("  "),
                    _ => format!("{:2}", (i - leading_spaces + 1)),
                };
                Widget::Label {
                    text,
                    id: None,
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
        panel.visible = false;
        if let Some(ref mut layer) = panel.layer {
            layer.set_size(1, 1);
        }
    }

    pub fn show(panel: &mut Panel) {
        panel.visible = true;
        if let Some(ref mut layer) = panel.layer {
            layer.set_size(240, 200);
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
