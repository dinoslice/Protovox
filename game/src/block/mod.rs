#[repr(u16)]
#[derive(Clone, Copy, Eq, PartialEq, Default)]
pub enum Block {
    #[default]
    Air = 0,
    Grass,
    Dirt,
}