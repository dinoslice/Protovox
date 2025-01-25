extern crate nalgebra_glm as glm;

mod logging;
mod run;

pub use {logging::init_tracing, run::run};