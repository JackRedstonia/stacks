use std::time::{Duration, Instant};

pub struct TimeState {
    start: Instant,
    last_update: Instant,
    elapsed: Duration,
}

impl TimeState {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start: now,
            last_update: now,
            elapsed: Duration::ZERO,
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        self.last_update = now;
        self.elapsed = now.duration_since(self.start);
    }

    pub fn start(&self) -> Instant {
        self.start
    }

    pub fn last_update(&self) -> Instant {
        self.last_update
    }

    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }
}
