use std::time::{Duration, Instant};

pub enum FrameMode {
    Idle { animation_cap: u32 },
    Cap(u32),
    Display, // reserved
}

pub struct Budget {
    pub mode: FrameMode,
    dirty: bool,
    // count of running animations; 0 = none
    animating: u32,
    // None = never painted
    last_paint: Option<Instant>,
}

impl Budget {
    pub fn new(mode: FrameMode) -> Self {
        Budget {
            mode,
            dirty: false,
            animating: 0,
            last_paint: None,
        }
    }

    pub fn on_frame(&mut self, now: Instant) -> Action {
        let needs_work = self.dirty || self.animating > 0;
        if !needs_work {
            return Action::Sleep;
        }

        match self.last_paint {
            Some(last) if now.duration_since(last) < self.interval() => Action::SkipKeepAlive,
            _ => {
                self.last_paint = Some(now);
                self.dirty = false;
                Action::Paint
            }
        }
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn interval(&self) -> Duration {
        match self.mode {
            FrameMode::Idle { animation_cap } | FrameMode::Cap(animation_cap) => {
                Duration::from_millis(1000 / animation_cap as u64)
            }
            // "always long enough" for now
            FrameMode::Display => Duration::ZERO,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Action {
    Paint,
    SkipKeepAlive,
    Sleep,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    // new(), no mark_dirty -> Sleep
    fn clean_idle_sleeps() {
        let mut b = Budget::new(FrameMode::Idle { animation_cap: 60 });
        let t0 = Instant::now();
        assert_eq!(b.on_frame(t0), Action::Sleep);
    }

    #[test]
    // mark_dirty, last=None -> Paint
    fn first_dirty_paints() {
        let mut b = Budget::new(FrameMode::Idle { animation_cap: 60 });

        b.mark_dirty();
        let t0 = Instant::now();
        assert_eq!(b.on_frame(t0), Action::Paint);
    }

    #[test]
    // paint once, dirty again, +5ms -> Skip
    fn dirty_before_interval_skips() {
        let mut b = Budget::new(FrameMode::Idle { animation_cap: 60 });
        let t0 = Instant::now();

        b.mark_dirty();
        assert_eq!(b.on_frame(t0), Action::Paint);

        b.mark_dirty();
        let t1 = t0 + Duration::from_millis(5);
        assert_eq!(b.on_frame(t1), Action::SkipKeepAlive);
    }

    #[test]
    // paint once, dirty again, +20ms -> Paint
    fn dirty_after_interval_paints() {
        let mut b = Budget::new(FrameMode::Idle { animation_cap: 60 });
        let t0 = Instant::now();

        b.mark_dirty();
        assert_eq!(b.on_frame(t0), Action::Paint);

        b.mark_dirty();
        let t1 = t0 + Duration::from_millis(20);
        assert_eq!(b.on_frame(t1), Action::Paint);
    }
}
