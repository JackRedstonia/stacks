use crate::prelude::*;

use std::time::Duration;

pub trait TimeReport: Widget {
    fn time(&mut self, progress: scalar);
}

// Transitions from one specific widget to its children permanently.
pub struct AB<A: TimeReport + ?Sized, B: Widget> {
    a: Option<(Wrap<A>, scalar)>,
    b: Wrap<B>,
    fade_time: Duration,
    running: Option<(Duration, Duration)>,
    just_switched: bool,
    size: Size,
    schedule_set_size: bool,
}

impl<A: TimeReport, B: Widget> AB<A, B> {
    pub fn new(a: Wrap<A>, b: Wrap<B>, fade_time: Duration) -> Wrap<Self> {
        Self {
            a: Some((a, 0.0)),
            b,
            fade_time,
            running: None,
            just_switched: false,
            size: Size::default(),
            schedule_set_size: false,
        }
        .into()
    }
}

impl<A: TimeReport, B: Widget> AB<A, B> {
    pub fn is_running(&self) -> bool {
        self.a.is_none() || self.running.is_some()
    }

    pub fn run(&mut self, duration: Duration) {
        if self.a.is_some() && self.running.is_none() {
            self.running = Some((duration, State::elapsed_draw()));
        }
    }

    fn tick_forward(&mut self) -> bool {
        if let Some((du, start)) = &self.running {
            let f = self.a.as_mut().unwrap();
            let delta = (State::elapsed_draw() - *start).as_secs_f32();
            let du = du.as_secs_f32();
            let fade_time = self.fade_time.as_secs_f32();
            let delta_scaled = delta / du;

            f.0.inner_mut().time(delta_scaled.min(1.0));
            if delta_scaled >= 1.0 {
                if f.1 == 0.0 {
                    self.schedule_set_size = true;
                }
                f.1 = ((delta - du) / fade_time).min(1.0);
            }

            let fin = delta > du + fade_time;
            if fin && self.a.is_some() {
                self.a = None;
                self.running = None;
            }
            return self.schedule_set_size;
        }
        false
    }
}

impl<A: TimeReport, B: Widget> Widget for AB<A, B> {
    fn load(&mut self, state: &mut WidgetState, stack: &mut ResourceStack) {
        if let Some((a, _)) = &mut self.a {
            a.load(stack);
        }
        self.b.load(stack);
    }

    fn update(&mut self, state: &mut WidgetState) {
        if let Some((a, _)) = &mut self.a {
            a.update();
        }
        self.b.update();
    }

    fn input(&mut self, state: &mut WidgetState, event: &InputEvent) -> bool {
        self.a
            .as_mut()
            .map(|a| a.0.input(event))
            .unwrap_or_else(|| self.b.input(event))
    }

    fn size(&mut self, state: &mut WidgetState) -> (LayoutSize, bool) {
        let mut changed = false;
        if let Some((a, f)) = &mut self.a {
            if *f == 0.0 {
                return a.size();
            }
            if self.just_switched {
                changed = true;
                self.just_switched = false;
            }
        }
        let (bs, bc) = self.b.size();
        (bs, bc || changed)
    }

    fn set_size(&mut self, state: &mut WidgetState, size: Size) {
        self.size = size;
        if let Some((a, f)) = &mut self.a {
            a.set_size(size);
            if *f != 0.0 {
                self.b.set_size(size);
            }
        } else {
            self.b.set_size(size);
        }
    }

    fn draw(&mut self, state: &mut WidgetState, canvas: &mut Canvas) {
        let t = self.tick_forward();
        self.just_switched |= t;
        if self.schedule_set_size {
            self.schedule_set_size = false;
            self.b.set_size(self.size);
        }
        match &mut self.a {
            Some((a, f)) => {
                if *f != 0.0 {
                    self.b.draw(canvas);
                    let i = canvas
                        .save_layer_alpha(None, ((1.0 - *f) * 255.0) as _);
                    a.draw(canvas);
                    canvas.restore_to_count(i);
                } else {
                    a.draw(canvas);
                }
            }
            None => {
                self.b.draw(canvas);
            }
        }
    }
}
