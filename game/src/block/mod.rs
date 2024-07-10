#[repr(u16)]
#[derive(Clone, Eq, PartialEq, Default)]
pub enum Block {
    #[default]
    Air = 0,
    Grass,
    Dirt,
}