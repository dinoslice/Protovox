use shipyard::{EntitiesViewMut, IntoIter, IntoWorkload, View, ViewMut, Workload};
use strck::IntoCk;
use dino_plugins::engine::{DinoEnginePlugin, EnginePluginMetadata};
use engine::components::{Hitbox, Transform};
use gizmos::{BoxGizmo, GizmoColor, GizmoLifetime, GizmoStyle};

pub struct VisualDebugPlugin;

impl DinoEnginePlugin for VisualDebugPlugin {
    fn late_update(&self) -> Option<Workload> {
        debug_draw_hitbox_gizmos
            .into_workload()
            .into()
    }

    fn plugin_metadata(&self) -> EnginePluginMetadata {
        EnginePluginMetadata {
            name: "visual_debug".ck().expect("valid identifier"),
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}

fn debug_draw_hitbox_gizmos(
    v_hitbox: View<Hitbox>,
    v_transform: View<Transform>,

    mut entities: EntitiesViewMut,
    mut vm_box_gizmos: ViewMut<BoxGizmo>,
) {
    for (transform, hitbox) in (&v_transform, &v_hitbox).iter() {
        let half_hitbox = hitbox.0 * 0.5;

        let min_extent = transform.position - half_hitbox;
        let max_extent = transform.position + half_hitbox;

        entities.add_entity(&mut vm_box_gizmos, BoxGizmo::from_corners(
            min_extent,
            max_extent,
            GizmoStyle::stroke(GizmoColor { r: 1.0, g: 0.0, b: 0.0 }),
            GizmoLifetime::SingleFrame,
        ));
    }
}