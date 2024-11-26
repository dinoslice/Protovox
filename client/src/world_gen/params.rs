use egui::{Response, Ui};

#[derive(Debug, Clone, PartialEq)]
pub struct WorldGenParams {
    pub continentalness_scale: f64,
    pub erosion_scale: f64,
    pub peaks_valleys_scale: f64,

    pub c_start: f32,
    pub c_end: f32,

    pub e_start: f32,
    pub e_end: f32,

    pub pv_start: f32,
    pub pv_end: f32,
}

impl Default for WorldGenParams {
    fn default() -> Self {
        Self {
            continentalness_scale: 0.01,
            erosion_scale: 0.02,
            peaks_valleys_scale: 0.03,
            c_start: -10.0,
            c_end: 150.0,
            e_start: -1.0,
            e_end: 1.0,
            pv_start: -30.0,
            pv_end: 30.0,
        }
    }
}

impl egui::Widget for &mut WorldGenParams {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("C Scale: ");
                ui.add(
                    egui::DragValue::new(&mut self.continentalness_scale)
                        .speed(0.01)
                        .custom_formatter(|f, _| format!("{f:.5}"))
                );
            });

            ui.horizontal(|ui| {
                ui.label("E Scale: ");
                ui.add(
                    egui::DragValue::new(&mut self.erosion_scale)
                        .speed(0.01)
                        .custom_formatter(|f, _| format!("{f:.5}"))
                );
            });

            ui.horizontal(|ui| {
                ui.label("PV Scale: ");
                ui.add(
                    egui::DragValue::new(&mut self.peaks_valleys_scale)
                        .speed(0.01)
                        .custom_formatter(|f, _| format!("{f:.4}"))
                );
            });

            ui.horizontal(|ui| {
                ui.label("Cont Start:");
                ui.add(egui::DragValue::new(&mut self.c_start));

                ui.label("Cont End:");
                ui.add(egui::DragValue::new(&mut self.c_end));
            });

            ui.horizontal(|ui| {
                ui.label("Erosion Start:");
                ui.add(egui::DragValue::new(&mut self.e_start));

                ui.label("Erosion End:");
                ui.add(egui::DragValue::new(&mut self.e_end));
            });

            ui.horizontal(|ui| {
                ui.label("PV Start:");
                ui.add(egui::DragValue::new(&mut self.pv_start));

                ui.label("PV End:");
                ui.add(egui::DragValue::new(&mut self.pv_end));
            });
        }).response
    }
}