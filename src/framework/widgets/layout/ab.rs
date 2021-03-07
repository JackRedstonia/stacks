use crate::prelude::*;

use std::time::Duration;

pub trait TimeReport: Widget {
    fn time(&mut self, progress: scalar);
}

// Transitions from one specific widget to its children permanently.
pub struct AB<A: TimeReport + ?Sized, B: Widget + ?Sized> {
    a: Option<(Wrap<A>, LayoutSize, scalar)>,
    b: Wrap<B>,
    b_size: LayoutSize,
    fade_time: Duration,
    running: Option<(Duration, Duration)>,
    just_switched: bool,
    size: Size,
    schedule_set_size: bool,
}

impl<A: TimeReport, B: Widget + ?Sized> AB<A, B> {
    pub fn new(a: Wrap<A>, b: Wrap<B>, fade_time: Duration) -> Wrap<Self> {
        FrameworkState::request_load();
        Self {
            a: Some((a, LayoutSize::default(), 0.0)),
            b,
            b_size: LayoutSize::default(),
            fade_time,
            running: None,
            just_switched: false,
            size: Size::default(),
            schedule_set_size: false,
        }
        .into()
    }
}

impl<A: TimeReport, B: Widget + ?Sized> AB<A, B> {
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
                if f.2 == 0.0 {
                    self.schedule_set_size = true;
                }
                f.2 = ((delta - du) / fade_time).min(1.0);
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

impl<A: TimeReport, B: Widget + ?Sized> Widget for AB<A, B> {
    fn load(&mut self, _state: &mut WidgetState, stack: &mut ResourceStack) {
        if let Some((a, _, _)) = &mut self.a {
            a.load(stack);
        }
        self.b.load(stack);
    }

    fn update(&mut self, _state: &mut WidgetState) {
        if let Some((a, _, _)) = &mut self.a {
            a.update();
        }
        self.b.update();
    }

    fn input(&mut self, _state: &mut WidgetState, event: &InputEvent) -> bool {
        self.a
            .as_mut()
            .map(|a| a.0.input(event))
            .unwrap_or_else(|| self.b.input(event))
    }

    fn size(&mut self, _state: &mut WidgetState) -> (LayoutSize, bool) {
        let mut changed = false;
        if let Some((a, s, f)) = &mut self.a {
            if *f == 0.0 {
                let sz = a.size();
                *s = sz.0;
                return sz;
            }
            if self.just_switched {
                changed = true;
                self.just_switched = false;
            }
        }
        let (bs, bc) = self.b.size();
        self.b_size = bs;
        (bs, bc || changed)
    }

    fn set_size(&mut self, _state: &mut WidgetState, size: Size) {
        self.size = size;
        if let Some((a, s, f)) = &mut self.a {
            a.set_size(s.layout_one(size));
            if *f != 0.0 {
                self.b.set_size(self.b_size.layout_one(size));
            }
        } else {
            self.b.set_size(self.b_size.layout_one(size));
        }
    }

    fn draw(&mut self, _state: &mut WidgetState, canvas: &mut Canvas) {
        let t = self.tick_forward();
        self.just_switched |= t;
        if self.schedule_set_size {
            self.schedule_set_size = false;
            self.b.set_size(self.size);
        }
        match &mut self.a {
            Some((a, _, f)) => {
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
