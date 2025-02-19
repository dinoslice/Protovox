use nalgebra_glm::Vec3;
use shipyard::{EntitiesViewMut, IntoIter, IntoWorkload, View, ViewMut, Workload};
use strck::IntoCk;
use dino_plugins::engine::{DinoEnginePlugin, EnginePluginMetadata};
use engine::components::{Hitbox, LocalPlayer, Transform};
use engine::VoxelEngine;
use game::chunk::CHUNK_SIZE;
use game::chunk::location::ChunkLocation;
use game::location::WorldLocation;
use gizmos::{BoxGizmo, GizmoColor, GizmoLifetime, GizmoStyle, GizmosPlugin, LineGizmo};

pub struct VisualDebugPlugin;

impl DinoEnginePlugin for VisualDebugPlugin {
    fn late_update(&self) -> Option<Workload> {
        (
            debug_draw_chunks,
            debug_draw_hitbox_gizmos
        )
            .into_workload()
            .into()
    }

    fn plugin_metadata(&self) -> EnginePluginMetadata {
        EnginePluginMetadata {
            name: "visual_debug".ck().expect("valid identifier"),
            version: env!("CARGO_PKG_VERSION"),
            dependencies: &[ &VoxelEngine, &GizmosPlugin ]
        }
    }
}

fn debug_draw_chunks(
    local_player: View<LocalPlayer>,
    v_transform: View<Transform>,

    mut entities: EntitiesViewMut,
    mut vm_line_gizmos: ViewMut<LineGizmo>,
) {
    // TODO: add egui window to change this

    let (transform, ..) = (&v_transform, &local_player).iter()
        .next()
        .expect("local player should exist");

    let mut lines = Vec::new();

    let start = WorldLocation::from(&transform.get_loc::<ChunkLocation>()).0;

    let dark_green = GizmoStyle::stroke(GizmoColor { r: 0.0, g: 0.8, b: 0.0 });

    let lifetime = GizmoLifetime::SingleFrame;

    let scale = 4;

    for axis in 0..3 {
        let cross = [(axis + 1) % 3, (axis + 2) % 3];

        let mut len = Vec3::zeros();
        len[axis] = CHUNK_SIZE[axis] as _;

        for i in 0..2 {
            let ca1 = cross[i];
            let ca2 = cross[i ^ 1];

            for c1 in (0..=CHUNK_SIZE[ca1]).step_by(scale) {
                let mut base = Vec3::zeros();
                base[ca1] = c1 as _;

                let start = start + base;

                lines.push(LineGizmo {
                    start,
                    end: start + len,
                    style: dark_green,
                    lifetime,
                });

                let mut c_len = Vec3::zeros();
                c_len[ca2] = CHUNK_SIZE[ca2] as _;

                let start = start + c_len;

                lines.push(LineGizmo {
                    start,
                    end: start + len,
                    style: dark_green,
                    lifetime,
                });
            }
        }
    }

    entities.bulk_add_entity(&mut vm_line_gizmos, lines);
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