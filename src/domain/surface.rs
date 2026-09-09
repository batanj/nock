use std::time::Instant;

use crate::{Action, Budget, FrameMode};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SurfaceId(u32);

pub struct Surface {
    id: SurfaceId,
    budget: Budget,
    // later: root: NodeId, size, scale
}

impl SurfaceId {
    pub fn new(n: u32) -> Self {
        SurfaceId(n)
    }
}

impl Surface {
    pub fn new(id: SurfaceId, mode: FrameMode) -> Self {
        Self {
            id,
            budget: Budget::new(mode),
        }
    }

    pub fn id(&self) -> SurfaceId {
        self.id
    }

    pub fn mark_dirty(&mut self) {
        self.budget.mark_dirty();
    }

    pub fn on_frame(&mut self, now: Instant) -> Action {
        self.budget.on_frame(now)
    }
}

#[cfg(test)]
mod tests {
    use crate::Action;

    use super::*;
    use std::time::Instant;

    #[test]
    fn new_surface_sleeps() {
        let mut s = Surface::new(SurfaceId::new(1), FrameMode::Idle { animation_cap: 60 });
        let t0 = Instant::now();
        assert_eq!(s.on_frame(t0), Action::Sleep);
    }

    #[test]
    fn mark_dirty_goes_to_budget() {
        let mut s = Surface::new(SurfaceId::new(1), FrameMode::Cap(60));
        s.mark_dirty();

        let t0 = Instant::now();
        assert_eq!(s.on_frame(t0), Action::Paint);
    }

    #[test]
    fn id_is_stable() {
        let s = Surface::new(SurfaceId::new(1), FrameMode::Cap(60));
        assert_eq!(s.id(), SurfaceId::new(1));
    }
}
