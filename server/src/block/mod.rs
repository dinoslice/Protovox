#[repr(u16)]
#[derive(Clone)]
pub enum Block {
    Air = 0,
    Grass,
    Dirt,
}