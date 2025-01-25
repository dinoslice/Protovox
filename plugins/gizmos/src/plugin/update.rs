use glm::Vec3;
use shipyard::{EntitiesViewMut, UniqueView, UniqueViewMut, ViewMut};
use engine::application::delta_time::LastDeltaTime;
use engine::rendering::graphics_context::GraphicsContext;
use crate::plugin::line_render_state::GizmosLineRenderState;
use crate::plugin::vertex::GizmoVertex;
use crate::types::{BoxGizmo, GizmoLifetime, LineGizmo};

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

pub fn decompose_box_gizmos(mut entities: EntitiesViewMut, mut vm_box_gizmos: ViewMut<BoxGizmo>, mut vm_line_gizmos: ViewMut<LineGizmo>) {
    for BoxGizmo { min, max, style, lifetime } in vm_box_gizmos.drain() {
        let corners = [
            Vec3::new(min.x, min.y, min.z),
            Vec3::new(min.x, min.y, max.z),
            Vec3::new(min.x, max.y, min.z),
            Vec3::new(min.x, max.y, max.z),
            Vec3::new(max.x, min.y, min.z),
            Vec3::new(max.x, min.y, max.z),
            Vec3::new(max.x, max.y, min.z),
            Vec3::new(max.x, max.y, max.z),
        ];

        let edges = [
            (0, 1), (0, 2), (1, 3), (2, 3), // bottom
            (4, 5), (4, 6), (5, 7), (6, 7), // top
            (0, 4), (1, 5), (2, 6), (3, 7), // vertical
        ];

        let map = edges.into_iter().map(|(start, end)| LineGizmo {
            start: corners[start],
            end: corners[end],
            style: style.clone(),
            lifetime: lifetime.clone(),
        });

        entities.bulk_add_entity(&mut vm_line_gizmos, map);
    }
}