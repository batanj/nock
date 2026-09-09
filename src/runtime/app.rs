use crate::domain::{Action, FrameMode, Surface, SurfaceId};
use std::collections::HashMap;
use std::time::Instant;

pub struct App {
    surfaces: HashMap<SurfaceId, Surface>,
    // Phase 1+: conn, qh, egl, proto maps...
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let surfaces = HashMap::new();
        App { surfaces }
    }

    pub fn add_surface(&mut self, id: SurfaceId, mode: FrameMode) {
        self.surfaces
            .entry(id)
            .or_insert_with(|| Surface::new(id, mode));
    }

    pub fn mark_dirty(&mut self, id: SurfaceId) -> bool {
        match self.surfaces.get_mut(&id) {
            None => false,
            Some(surface) => {
                surface.mark_dirty();
                true
            }
        }
    }

    pub fn on_frame(&mut self, id: SurfaceId, now: Instant) -> Option<Action> {
        self.surfaces
            .get_mut(&id)
            .map(|surface| surface.on_frame(now))
    }

    pub fn tick(&mut self, now: Instant) -> Vec<(SurfaceId, Action)> {
        self.surfaces
            .iter_mut()
            .map(|(&surface_id, surface)| (surface_id, Surface::on_frame(surface, now)))
            .collect()
    }

    pub fn remove_surface(&mut self, id: SurfaceId) -> bool {
        self.surfaces.remove(&id).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_id_does_not_paint() {
        let mut app = App::default();
        let now = Instant::now();

        assert_eq!(app.on_frame(SurfaceId::new(0), now), None)
    }

    #[test]
    fn added_surface_sleeps() {
        let mut app = App::default();
        let now = Instant::now();

        let id = SurfaceId::new(1);
        app.add_surface(id, FrameMode::Cap(60));
        assert_eq!(app.on_frame(id, now), Some(Action::Sleep));
    }

    #[test]
    fn dirty_one_not_the_other() {
        let mut app = App::default();
        let now = Instant::now();

        let id1 = SurfaceId::new(1);
        let id2 = SurfaceId::new(2);
        app.add_surface(id1, FrameMode::Cap(60));
        app.add_surface(id2, FrameMode::Cap(60));
        app.mark_dirty(id1);

        assert_eq!(app.on_frame(id1, now), Some(Action::Paint));
        assert_eq!(app.on_frame(id2, now), Some(Action::Sleep));
    }

    #[test]
    fn mark_unknown_is_false() {
        let mut app = App::default();
        assert!(!app.mark_dirty(SurfaceId::new(1)));
    }

    #[test]
    fn tick_empty_is_empty() {
        let mut app = App::default();
        let now = Instant::now();

        assert_eq!(app.tick(now), []);
    }

    #[test]
    fn tick_reports_each_surface() {
        let mut app = App::default();
        let now = Instant::now();

        let id1 = SurfaceId::new(1);
        let id2 = SurfaceId::new(2);
        app.add_surface(id1, FrameMode::Cap(60));
        app.add_surface(id2, FrameMode::Cap(60));
        app.mark_dirty(id1);

        let got = app.tick(now);

        assert!(got.contains(&(id1, Action::Paint)));
        assert!(got.contains(&(id2, Action::Sleep)));
        assert_eq!(got.len(), 2);
    }

    #[test]
    fn remove_then_unkown() {
        let mut app = App::default();
        let now = Instant::now();

        let id = SurfaceId::new(1);
        app.add_surface(id, FrameMode::Cap(60));

        assert!(app.remove_surface(id));
        assert_eq!(app.on_frame(id, now), None);
        assert!(!app.remove_surface(id));
    }
}
