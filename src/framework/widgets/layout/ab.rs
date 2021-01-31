use crate::prelude::*;

use std::time::Duration;

pub trait TimeReport: Widget {
    fn time(&mut self, progress: scalar);
}

// Transitions from one widget to another permanently.
pub struct AB<T: TimeReport, U: Widget> {
    a: Option<(Wrap<T>, scalar)>,
    b: Wrap<U>,
    running: Option<(Duration, Duration)>,
    just_switched: bool,
}

impl<T: TimeReport, U: Widget> AB<T, U> {
    pub fn new(a: impl Into<Wrap<T>>, b: impl Into<Wrap<U>>) -> Self {
        Self {
            a: Some((a.into(), 0.0)),
            b: b.into(),
            running: None,
            just_switched: false,
        }
    }

    pub fn is_running(&self) -> bool {
        self.a.is_some() && self.running.is_none()
    }

    pub fn run(&mut self, duration: Duration) {
        if self.is_running() {
            self.running = Some((duration, State::elapsed_draw()));
        }
    }

    fn tick_forward(&mut self) -> bool {
        if let Some((du, start)) = &self.running {
            let f = self.a.as_mut().unwrap();
            let now = State::elapsed_draw();
            let delta = (now - *start).as_secs_f32() / du.as_secs_f32();

            f.0.inner().time(delta);

            let fin = delta >= 1.0;
            return fin;
        }
        false
    }
}

impl<T: TimeReport, U: Widget> Widget for AB<T, U> {
    fn load(&mut self, _wrap: &mut WidgetState, stack: &mut ResourceStack) {
        if let Some((a, _)) = &mut self.a {
            a.load(stack);
        }
        self.b.load(stack);
    }

    fn update(&mut self, _wrap: &mut WidgetState) {
        if let Some((a, _)) = &mut self.a {
            a.update();
        }
        self.b.update();
    }

    fn input(&mut self, _wrap: &mut WidgetState, event: &InputEvent) -> bool {
        if let Some((a, _)) = &mut self.a {
            a.input(event)
        } else {
            self.b.input(event)
        }
    }

    fn size(&mut self, _wrap: &mut WidgetState) -> (LayoutSize, bool) {
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

    fn set_size(&mut self, _wrap: &mut WidgetState, size: Size) {
        if let Some((a, _)) = &mut self.a {
            a.set_size(size);
        } else {
            self.b.set_size(size);
        }
    }

    fn draw(&mut self, _wrap: &mut WidgetState, canvas: &mut Canvas) {
        self.just_switched |= self.tick_forward();
        match &mut self.a {
            Some((a, f)) => {
                a.draw(canvas);
                if *f != 0.0 {
                    let i = canvas.save_layer_alpha(None, (*f * 255.0) as _);
                    self.b.draw(canvas);
                    canvas.restore_to_count(i);
                }
            }
            None => {
                self.b.draw(canvas);
            }
        }
    }
}
