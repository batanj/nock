use std::{error::Error, time::Duration};

use calloop::{EventLoop, timer::Timer};
use calloop_wayland_source::WaylandSource;
use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    registry::RegistryState,
    shell::{
        WaylandSurface,
        wlr_layer::{Anchor, Layer, LayerShell},
    },
    shm::{Shm, slot::SlotPool},
};
use wayland_client::{Connection, globals::registry_queue_init};

use crate::{
    Action::Paint,
    FrameMode, SurfaceId,
    runtime::{app::App, nock::Nock},
};

pub fn run() -> Result<(), Box<dyn Error>> {
    let conn = Connection::connect_to_env()?;
    let (globals, event_queue) = registry_queue_init::<Nock>(&conn)?;
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh)?;
    let layer_shell = LayerShell::bind(&globals, &qh)?;
    let shm = Shm::bind(&globals, &qh)?;
    let surface = compositor.create_surface(&qh);
    let layer = layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("nock"), None);
    layer.set_anchor(Anchor::TOP | Anchor::LEFT | Anchor::RIGHT);
    layer.set_exclusive_zone(32);
    layer.set_size(0, 32);
    layer.wl_surface().commit();

    let pool = SlotPool::new(64 * 64 * 4, &shm)?;

    let mut app = App::new();
    let id = SurfaceId::new(1);
    app.add_surface(id, FrameMode::Idle { animation_cap: 60 });

    let mut nock = Nock {
        app,
        clock_id: id,
        registry_state: RegistryState::new(&globals),
        compositor_state: compositor,
        exit: false,
        layer_shell,
        output_state: OutputState::new(&globals, &qh),
        layer_surface: layer,
        pool,
        shm,
        width: 0,
        height: 32,
        red: true,
    };

    let mut event_loop = EventLoop::<Nock>::try_new()?;
    let handle = event_loop.handle();
    handle.insert_source(
        Timer::from_duration(Duration::from_secs(1)),
        |deadline, _, nock| {
            nock.app.mark_dirty(nock.clock_id);
            for (id, action) in nock.app.tick(deadline) {
                if action == Paint && id == nock.clock_id {
                    nock.red = !nock.red;
                    nock.draw();
                    println!("{id:?} -> {action:?} red={}", nock.red);
                }
            }
            calloop::timer::TimeoutAction::ToDuration(Duration::from_secs(1))
        },
    )?;

    WaylandSource::new(conn, event_queue).insert(event_loop.handle())?;

    event_loop.run(None, &mut nock, |_| {})?;

    Ok(())
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
