extern crate winit;

use winit::event_loop::{ControlFlow, EventLoop};
use instance::EngineInstance;

mod instance;
mod renderer;

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create window event loop.");
    
    event_loop.set_control_flow(ControlFlow::Poll);
    
    let mut app = EngineInstance::default();
    event_loop.run_app(&mut app)
        .expect("Failed to run application.");
}