extern crate nalgebra as na;
extern crate nalgebra_glm as glm;

mod render;
pub mod input;

use std::str::FromStr;
use pollster::FutureExt;
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use tracing_subscriber::util::{SubscriberInitExt, TryInitError};

fn init_tracing() -> Result<(), TryInitError>{
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_str("debug,wgpu=warn,wgpu_core=warn,wgpu_hal=warn,naga=warn").expect("setting filter failed"))
        .finish();
    subscriber.try_init()
}

fn main() {
    init_tracing().expect("tracing initialized");

    render::run();
}
