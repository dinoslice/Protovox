extern crate nalgebra as na;
extern crate nalgebra_glm as glm;

mod render;
pub mod input;

use std::str::FromStr;
use pollster::FutureExt;
use tracing::debug;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoopBuilder};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowBuilder};

fn init_tracing() {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_str("DEBUG").expect("setting filter failed"))
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");
}

fn main() {
    init_tracing();

    render::run();
}
