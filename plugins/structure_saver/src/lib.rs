use egui::{Response, Ui, Widget};
use game::location::BlockLocation;

pub struct StructureSaver(Option<StructureBuilder>);

pub struct StructureBuilder {
    start: BlockLocation,
    end: BlockLocation,
    intended_save: String,
}

impl Widget for StructureSaver {
    fn ui(self, ui: &mut Ui) -> Response {
        todo!()
    }
}
