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

            ui.label("Camera Offset:");
            vec_edit_horizontal(ui, &mut self.cam_offset.0, 0.1);

            // Modify `request_reframe`
            ui.checkbox(&mut self.lock_position, "Lock Position");
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