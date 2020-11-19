mod application;

use std::ffi::CString;

use skulpin_renderer::PresentMode;
use application::{Application, ApplicationBuilder, input::InputEvent};

fn main() {
    ApplicationBuilder::new()
        .app_name(CString::new("Stacks").unwrap())
        .window_title("Stacks")
        .present_mode_priority(vec![PresentMode::Immediate])
        .run(Stacks {})
}

struct Stacks {}

impl Application for Stacks {
    fn update(&mut self, input_state: (), time_state: ()) {
        
    }

    fn draw(&mut self, input_state: (), time_state: (), canvas: ()) {
        
    }

    fn input(&mut self, input_state: (), time_state: (), event: InputEvent) {
        
    }

    fn close(&mut self) {
        
    }

    fn crash(&mut self, err: application::ApplicationError) {
        println!("Stacks has crashed!\nMore info: {:?}", err);
    }
}
