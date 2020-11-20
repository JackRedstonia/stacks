use super::application::{
    input::{InputEvent, InputState},
    time::TimeState,
    Application, ApplicationError,
};
use super::canvas::Canvas;
use super::components::Component;

pub struct Stacks<C: Component> {
    root: C,
}

impl<T: Component> Stacks<T> {
    pub fn new(root: T) -> Self {
        Self { root }
    }
}

impl<T: Component> Application for Stacks<T> {
    fn update(&mut self, input_state: &InputState, time_state: &TimeState) {
        self.root.update(input_state, time_state);
    }

    fn draw(&mut self, input_state: &InputState, time_state: &TimeState, canvas: &mut Canvas) {
        self.root.draw(input_state, time_state, canvas);
    }

    fn input(&mut self, input_state: &InputState, time_state: &TimeState, event: InputEvent) {
        self.root.input(input_state, time_state, &event);
    }

    fn close(&mut self) {}

    fn crash(&mut self, err: ApplicationError) {
        println!("Stacks has crashed!\nMore info: {:?}", err);
    }
}
