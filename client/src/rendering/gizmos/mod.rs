use shipyard::{IntoWorkload, Workload};
use wgpu::util::DeviceExt;
use settings::read_settings;
use line_render_state::initialize_line_gizmos_render_state;

pub mod types;
pub use types::*;

pub(super) mod line_render_state;
mod settings;
mod vertex;

pub fn initialize() -> Workload {
    (
        read_settings,
        initialize_line_gizmos_render_state,
    ).into_sequential_workload()
}

