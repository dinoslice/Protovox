use egui::Context;
use shipyard::Unique;

#[derive(Unique)]
pub struct CurrentEguiFrame(pub(crate) Context);

impl CurrentEguiFrame {
    pub fn ctx(&self) -> &Context {
        &self.0
    }
}