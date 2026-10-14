use crate::error::AppError;
use crate::format::brightness_text;
use crate::runtime::panel::Panel;
use crate::runtime::{bar::Bar, menu::Menu};
use std::{
    error::Error,
    ffi::c_void,
    time::{Duration, Instant},
};

use calloop::{EventLoop, channel, timer::Timer};
use calloop_wayland_source::WaylandSource;
use smithay_client_toolkit::{
    compositor::CompositorState, output::OutputState, registry::RegistryState, seat::SeatState,
    shell::wlr_layer::LayerShell,
};
use wayland_client::{Connection, globals::registry_queue_init};

use crate::{
    Action::Paint,
    FrameMode, SurfaceId, Widget,
    format::{battery_text, format_date, format_now, network_text, volume_text},
    runtime::{app::App, nock::Nock},
    theme::Theme,
};

pub fn run() -> Result<(), Box<dyn Error>> {
    let conn = Connection::connect_to_env()?;

    let display_ptr = conn.backend().display_ptr() as *mut c_void;

    let (globals, event_queue) = registry_queue_init::<Nock>(&conn)?;
    let qh = event_queue.handle();
    let compositor = CompositorState::bind(&globals, &qh)?;

    let mut app = App::new();

    let bar_surface_id = SurfaceId::new(1);
    app.add_surface(bar_surface_id, FrameMode::Idle { animation_cap: 60 });

    // init bar
    let layer_shell = LayerShell::bind(&globals, &qh)?;
    let bar_layer_surface = Bar::create(&layer_shell, &compositor, &qh)?;

    // init panel
    let panel_surface_id = SurfaceId::new(2);
    app.add_surface(panel_surface_id, FrameMode::Idle { animation_cap: 60 });
    let panel_layer_surface = Some(Panel::create(&layer_shell, &compositor, &qh)?);

    // init menu
    let menu_surface_id = SurfaceId::new(3);
    app.add_surface(menu_surface_id, FrameMode::Idle { animation_cap: 60 });
    let menu_layer_surface = Some(Menu::create(&layer_shell, &compositor, &qh)?);

    let mut nock = Nock {
        app,
        clock_id: bar_surface_id,
        registry_state: RegistryState::new(&globals),
        compositor_state: compositor,
        exit: false,
        layer_shell,
        output_state: OutputState::new(&globals, &qh),
        seat_state: SeatState::new(&globals, &qh),
        display_ptr,
        bar: Bar::new(
            bar_surface_id,
            Some(bar_layer_surface),
            None,
            0,
            32,
            vec![],
            Theme::dark(),
        ),
        panel: Some(Panel::new(
            panel_layer_surface,
            None,
            0,
            0,
            panel_surface_id,
            Theme::dark(),
        )),
        menu: Some(Menu::new(
            menu_layer_surface,
            None,
            0,
            0,
            menu_surface_id,
            Theme::dark(),
        )),
        is_dark: true,
        hour12: true,
        last_net: Instant::now(),
    };

    let mut event_loop = EventLoop::<Nock>::try_new()?;

    clock(&mut event_loop);

    WaylandSource::new(conn, event_queue).insert(event_loop.handle())?;

    event_loop.run(None, &mut nock, |_| {})?;

    Ok(())
}

pub fn clock(event_loop: &mut EventLoop<Nock>) {
    let handle = event_loop.handle();

    let (sender, channel_source) = channel::channel::<String>();

    handle
        .insert_source(channel_source, |event, _, nock| {
            if let channel::Event::Msg(network_string) = event {
                Widget::set_network(&mut nock.bar.root, &network_string);
                nock.app.mark_dirty(nock.clock_id);
                nock.bar.draw();
            }
        })
        .expect("Failed to insert channel source");

    handle
        .insert_source(
            Timer::from_duration(Duration::from_secs(1)),
            move |deadline, _, nock| {
                nock.bar.reload_if_changed();
                if let Some(menu) = nock.menu.as_mut() {
                    menu.reload_if_changed();
                }

                Widget::set_clock(&mut nock.bar.root, &format_now(nock.hour12));

                if let Some(panel) = &nock.panel
                    && panel.selected_day.is_none()
                {
                    Widget::set_date(&mut nock.bar.root, &format_date());
                }

                let battery =
                    AppError::log_and_fallback(battery_text(), "battery", "󰂑".to_string());
                Widget::set_battery(&mut nock.bar.root, &battery);

                let volume =
                    AppError::log_and_fallback(volume_text(), "volume", "󰖁 Muted".to_string());
                Widget::set_volume(&mut nock.bar.root, &volume);

                let brightness =
                    AppError::log_and_fallback(brightness_text(), "brightness", "0".to_string());
                Widget::set_brightness(&mut nock.bar.root, &brightness);

                if Instant::now().duration_since(nock.last_net) > Duration::from_secs(10) {
                    nock.last_net = Instant::now();

                    let sender_clone = sender.clone();

                    std::thread::spawn(move || {
                        let text =
                            AppError::log_and_fallback(network_text(), "network", "󰖪".to_string());
                        let _ = sender_clone.send(text);
                    });
                }

                nock.app.mark_dirty(nock.clock_id);

                for (id, action) in nock.app.tick(deadline) {
                    if action == Paint && id == nock.clock_id {
                        nock.bar.draw();
                    }
                }
                calloop::timer::TimeoutAction::ToDuration(Duration::from_secs(1))
            },
        )
        .expect("Failed to insert source");
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc, time::Duration};

    use calloop::timer::{TimeoutAction, Timer};

    use crate::Action;

    use super::*;

    #[test]
    fn test_calloop_dispatch() {
        let fired = Rc::new(Cell::new(false));
        let fired_cb = Rc::clone(&fired);

        let mut app = App::new();
        let id = SurfaceId::new(1);
        app.add_surface(id, FrameMode::Idle { animation_cap: 60 });

        let mut event_loop = EventLoop::<App>::try_new().expect("Failed to initialize event loop");

        let handle = event_loop.handle();
        handle
            .insert_source(Timer::immediate(), move |deadline, _meta, app| {
                app.mark_dirty(id);
                let got = app.tick(deadline);
                assert!(got.contains(&(id, Action::Paint)));
                fired_cb.set(true);
                TimeoutAction::Drop
            })
            .expect("Failed to insert source");

        event_loop
            .dispatch(Some(Duration::from_millis(100)), &mut app)
            .expect("Failed to dispatch event loop");
        assert!(fired.get(), "timer callback never ran");
    }
}
