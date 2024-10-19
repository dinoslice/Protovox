use shipyard::{IntoWorkload, UniqueView, UniqueViewMut, ViewMut, Workload};
use wgpu::util::DeviceExt;
use settings::read_settings;
use line_render_state::initialize_line_gizmos_render_state;
use line_render_state::GizmosLineRenderState;
use vertex::GizmoVertex;

pub mod types;
pub use types::*;
use crate::application::delta_time::LastDeltaTime;
use crate::rendering::graphics_context::GraphicsContext;

pub(super) mod line_render_state;
mod settings;
mod vertex;

pub fn initialize() -> Workload {
    (
        read_settings,
        initialize_line_gizmos_render_state,
    ).into_sequential_workload()
}

pub fn update() -> Workload {
    (
        process_line_gizmos,
    ).into_workload()
}

pub fn process_line_gizmos(g_ctx: UniqueView<GraphicsContext>, mut line_gizmo_rend_state: UniqueViewMut<GizmosLineRenderState>, mut vm_line_gizmos: ViewMut<LineGizmo>, delta_time: UniqueView<LastDeltaTime>) {
    let mut buf = Vec::with_capacity(vm_line_gizmos.len() * 2);

    vm_line_gizmos.retain_mut(|_, mut g| {
        let color = bytemuck::cast(g.style.stroke_color);

        buf.push(GizmoVertex { position: *g.start.as_ref(), color });
        buf.push(GizmoVertex { position: *g.end.as_ref(), color });

        match &mut g.lifetime {
            GizmoLifetime::SingleFrame => false,
            GizmoLifetime::Persistent(t) => match t.checked_sub(delta_time.0) {
                None => false,
                Some(rem) => {
                    *t = rem;
                    true
                }
            }
        }
    });

    line_gizmo_rend_state.update_buffer(&g_ctx, &buf);
}