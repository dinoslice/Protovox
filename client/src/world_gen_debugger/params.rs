use egui::{Response, Ui};
use glm::TVec3;
use shipyard::{IntoWorkloadSystem, Unique};
use game::chunk::location::ChunkLocation;
use game::location::WorldLocation;
use crate::render_distance::RenderDistance;

#[derive(Unique, Debug, Clone)]
pub struct WorldGenVisualizerParams {
    pub generate_center: ChunkLocation,
    pub render_distance: RenderDistance,

    pub cam_offset: WorldLocation,
    pub lock_position: bool,
    pub auto_target_camera: bool,

    pub req_guess: bool,
}

impl egui::Widget for &mut WorldGenVisualizerParams {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.label("Generation Center:");
            vec_edit_horizontal(ui, &mut self.generate_center.0, 0.5);

            ui.label("Render Distance:");
            ui.horizontal(|ui| {
                ui.label("xz:");

                if ui.add(egui::DragValue::new(&mut self.render_distance.0.x)).changed() {
                    self.render_distance.0.z = self.render_distance.0.x;
                }

                ui.label("y:");
                ui.add(egui::DragValue::new(&mut self.render_distance.0.y));
            });

            ui.horizontal(|ui| {
                ui.label("Camera Offset:");

                if ui.button("Guess").clicked() {
                    self.req_guess = true;
                }
            });

            vec_edit_horizontal(ui, &mut self.cam_offset.0, 0.1);

            // Modify `request_reframe`
            ui.horizontal(|ui| {
                if ui.checkbox(&mut self.lock_position, "Lock Position").changed() {
                    if !self.lock_position {
                        self.auto_target_camera = false;
                    }
                }

                if self.lock_position {
                    ui.checkbox(&mut self.auto_target_camera, "Auto Target Camera");
                }
            })
        }).response
    }
}

fn vec_edit_horizontal<T: egui::emath::Numeric>(ui: &mut Ui, vec: &mut TVec3<T>, speed: f64) {
    ui.horizontal(|ui| {
        ui.label("x:");
        ui.add(egui::DragValue::new(&mut vec[0]).speed(speed));
        ui.label("y:");
        ui.add(egui::DragValue::new(&mut vec[1]).speed(speed));
        ui.label("z:");
        ui.add(egui::DragValue::new(&mut vec[2]).speed(speed));
    });
}