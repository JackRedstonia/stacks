use super::components::Component;
use super::game::Canvas;
use super::game::{Game, GameError, InputEvent, InputState, TimeState};

pub struct Stacks<T: Component> {
    root: T,
}

impl<T: Component> Stacks<T> {
    pub fn new(root: T) -> Self {
        Self { root }
    }
}

impl<T: Component> Game for Stacks<T> {
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

    fn crash(&mut self, err: GameError) {
        println!("Stacks has crashed!\nMore info: {:?}", err);
    }
}
