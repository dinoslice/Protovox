#[repr(u8)]
#[derive(Copy, Clone, Debug, Default, strum::Display, strum::VariantArray)]
#[strum(serialize_all = "snake_case")]
pub enum TextureId {
    Grass = 0,
    GrassSide,
    Dirt,
    
    Stone,
    Cobblestone,
    
    LogSide,
    LogTop,
    Planks,
    
    CrateSide,
    CrateTop,
    CrateBottom,
    
    Leaves,
    Water,
    
    DebugRed,
    DebugGreen,
    DebugBlue,
    
    Selection,
    #[default]
    Missing,
}