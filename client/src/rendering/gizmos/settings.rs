use shipyard::{AllStoragesView, Unique};

#[derive(Unique)]
pub struct GizmoRenderingSettings {
    pub max_line_gizmos: u16,
    pub max_box_gizmos: u16,
}

impl Default for GizmoRenderingSettings {
    fn default() -> Self {
        Self {
            max_line_gizmos: 512,
            max_box_gizmos: 512,
        }
    }
}

impl GizmoRenderingSettings {
    pub fn num_lines(&self) -> u32 {
        self.max_line_gizmos as u32 + self.max_box_gizmos as u32 * 12
    }
}

pub fn read_settings(storages: AllStoragesView) {
    // TODO: parse from file somewhere else
    storages.add_unique(GizmoRenderingSettings::default())
}