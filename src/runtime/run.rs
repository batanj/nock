use std::{error::Error, time::Duration};

use calloop::{
    EventLoop,
    timer::{TimeoutAction, Timer},
};

use crate::{FrameMode, SurfaceId, runtime::app::App};

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut app = App::new();
    let id = SurfaceId::new(1);
    app.add_surface(id, FrameMode::Idle { animation_cap: 60 });

    let mut event_loop = EventLoop::<App>::try_new()?;
    let handle = event_loop.handle();
    handle.insert_source(
        Timer::from_duration(Duration::from_secs_f32(1.0)),
        |deadline, _meta, app| {
            app.mark_dirty(id);
            for (sid, action) in app.tick(deadline) {
                println!("{sid:?} -> {action:?}");
            }
            TimeoutAction::ToDuration(Duration::from_secs(1))
        },
    )?;

    event_loop.run(None, &mut app, |_| {})?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

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
