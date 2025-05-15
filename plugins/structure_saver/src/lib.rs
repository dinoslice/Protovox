extern crate nalgebra_glm as glm;

use std::fs;
use std::path::Path;
use egui::{DragValue, Response, Ui, Widget, Window};
use egui::emath::Numeric;
use glm::{TVec, TVec3, Vec3};
use shipyard::{AllStoragesView, EntitiesView, EntitiesViewMut, IntoWorkload, Unique, UniqueView, UniqueViewMut, ViewMut, Workload};
use strck::IntoCk;
use dino_plugins::engine::{DinoEnginePlugin, EnginePluginMetadata};
use egui_systems::{CurrentEguiFrame, DuringEgui, EguiSystemsPlugin};
use engine::chunks::chunk_manager::ChunkManager;
use engine::structures::Structure;
use engine::VoxelEngine;
use game::block::Block;
use game::chunk::pos::ChunkPos;
use game::location::BlockLocation;
use gizmos::{BoxGizmo, GizmoColor, GizmoLifetime, GizmoStyle};

pub struct StructureSaverPlugin;

impl DinoEnginePlugin for StructureSaverPlugin {
    fn early_startup(&self) -> Option<Workload> {
        (
            |storages: AllStoragesView| storages.add_unique(StructureSaver(None)),
        )
            .into_sequential_workload()
            .into()
    }

    fn early_update(&self) -> Option<Workload> {
        (
            save_structure,
            gizmos,
        )
            .into_sequential_workload()
            .into()
    }

    fn render(&self) -> Option<Workload> {
        (
            |mut saver: UniqueViewMut<StructureSaver>, storages: AllStoragesView, frame: UniqueView<CurrentEguiFrame>|
                Window::new("Structure Saver")
                    .default_open(true)
                    .show(frame.ctx(), |ui| ui.add(StructureSaverUi { saver: &mut saver, storages }))
        )
            .into_sequential_workload()
            .order_egui()
            .into()
    }

    fn plugin_metadata(&self) -> EnginePluginMetadata {
        EnginePluginMetadata {
            name: "structure_saver_plugin".ck().expect("valid identifier"),
            version: "0.1.0",
            dependencies: &[
                &VoxelEngine,
                &EguiSystemsPlugin,
            ],
        }
    }
}

#[derive(Unique)]
pub struct StructureSaver(Option<StructureBuilder>);

#[derive(Default, Debug, Clone)]
pub struct StructureBuilder {
    start: BlockLocation,
    end: BlockLocation,
    origin: BlockLocation,
    intended_save: String,
}

pub struct StructureSaverUi<'a> {
    pub saver: &'a mut StructureSaver,
    pub storages: AllStoragesView<'a>,
}

#[derive(Unique)]
struct StructureSaveEvent;

impl Widget for StructureSaverUi<'_> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        // Temporarily take the builder out
        let mut builder = self.saver.0.take();

        let response = if let Some(mut builder_inner) = builder {
            let mut should_cancel = false;
            let mut should_save = false;

            let response = ui.vertical(|ui| {
                let mut resp = ui.label("New Structure");

                resp |= ui.add(VecEdit { vec: &mut builder_inner.start.0, name: "Start: ", speed: 0.5 });
                resp |= ui.add(VecEdit { vec: &mut builder_inner.end.0, name: "End: ", speed: 0.5 });
                resp |= ui.add(VecEdit { vec: &mut builder_inner.origin.0, name: "Origin: ", speed: 0.5 });

                resp |= ui.horizontal(|ui| {
                    let edit_resp = ui.text_edit_singleline(&mut builder_inner.intended_save);
                    let button_resp = ui.button("Save");

                    if button_resp.clicked() {
                        should_save = true;
                    }

                    edit_resp | button_resp
                }).response;

                let button_resp = ui.button("Cancel");

                if button_resp.clicked() {
                    should_cancel = true;
                }

                resp | button_resp
            }).response;

            // Put it back unless cancelled
            if !should_cancel {
                self.saver.0 = Some(builder_inner);
            }

            if should_save {
                self.storages.add_unique(StructureSaveEvent);
            }

            response
        } else {
            let resp = ui.label("No structure");
            let button_resp = ui.button("New structure");

            if button_resp.clicked() {
                self.saver.0 = Some(StructureBuilder::default());
            }

            resp | button_resp
        };

        response
    }
}


fn save_structure(storages: AllStoragesView, mut saver: UniqueViewMut<StructureSaver>, world: UniqueView<ChunkManager>) {
    let Ok(StructureSaveEvent) = storages.remove_unique() else {
        return;
    };

    let Some(saver) = &mut saver.0 else {
        tracing::error!("StructureSaver should have some if event was sent");
        return;
    };

    let min = glm::min2(&saver.start.0, &saver.end.0);
    let max = glm::max2(&saver.start.0, &saver.end.0);

    let Some(size) = (max - min).try_cast::<u8>() else {
        tracing::warn!("Structure must be smaller than 32x64x32");
        return;
    };

    let Some(mut structure) = (saver.origin.0 - min)
        .try_cast::<u8>()
        .map(|v| v.try_into().ok())
        .flatten()
        .map(|o| Structure::new(size, o))
        .flatten()
    else {
        tracing::warn!("Origin must be within structure bounds");
        return;
    };

    let size_end = structure.size();

    for x in 0..size_end.x() {
        for y in 0..size_end.y() {
            for z in 0..size_end.z() {
                let pos = ChunkPos::new(x, y, z).expect("should be within range");

                let block = structure.get_mut(pos).expect("should be valid index");

                let wp = BlockLocation(min + pos.as_vec().cast());

                *block = world.get_block_ref(&wp).cloned().unwrap_or_else(|| {
                    tracing::warn!("Failed to get block at {wp:?} when saving block");

                    Block::Air
                });
            }
        }
    }

    let bytes = match postcard::to_allocvec(&structure) {
        Ok(bytes) => bytes,
        Err(err) => {
            tracing::error!("Failed to serialize structure: {err:?}");
            return;
        }
    };

    if let Err(err) = fs::write(&saver.intended_save, bytes) {
        tracing::error!("Failed to save structure to file at {:?}: {err}", saver.intended_save);
        return;
    }

    tracing::info!("successfully saved");
}

struct VecEdit<'a, T: Numeric, const N: usize> {
    pub vec: &'a mut TVec<T, N>,
    pub name: &'static str,
    pub speed: f64,
}

impl<T: Numeric, const N: usize> Widget for VecEdit<'_, T, N> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            ui.label(self.name);

            for i in 0..N {
                ui.add(DragValue::new(&mut self.vec[i]).speed(self.speed));
            }
        })
            .response
    }
}

fn gizmos(saver: UniqueView<StructureSaver>, mut entities: EntitiesViewMut, mut vm_box_gizmo: ViewMut<BoxGizmo>) {
    let Some(builder) = &saver.0 else {
        return;
    };

    entities.add_entity(&mut vm_box_gizmo, BoxGizmo::from_corners(
        builder.start.0.cast(),
        builder.end.0.cast(),
        GizmoStyle::stroke(GizmoColor::new(0.9, 0.9, 0.2)),
        GizmoLifetime::SingleFrame,
    ));

    let origin = builder.origin.0.cast();

    entities.add_entity(&mut vm_box_gizmo, BoxGizmo::from_corners(
        origin,
        origin + Vec3::from_element(1.0),
        GizmoStyle::stroke(GizmoColor::new(0.9, 0.9, 0.2)),
        GizmoLifetime::SingleFrame,
    ));
}