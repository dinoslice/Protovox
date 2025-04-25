use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

#[derive(Default)]
pub struct EngineInstance {
    window: Option<Arc<Window>>,
}

impl ApplicationHandler for EngineInstance {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = event_loop.create_window(WindowAttributes::default()).ok().and_then(|window| Some(Arc::new(window)));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let Some(window)= self.window.as_ref() else { return; };
        if window.id() != window_id { return; }
        
        match event {
            WindowEvent::CloseRequested => {
                println!("Exiting");
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                window.request_redraw();
            }
            _ => {}
        }
    }
}