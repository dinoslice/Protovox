use std::fmt;
use glm::TVec3;
use crate::chunk::CHUNK_SIZE;
use crate::location::{BlockLocation, WorldLocation};

macro_rules! impl_getters_setters {
    ($axis:ident, $set_axis:ident, $set_axis_unchecked:ident, $offset:expr, $max:expr) => {
        pub fn $set_axis(&mut self, $axis: u8) -> Result<(), ChunkCoordOutOfRange> {
            if $axis > $max {
                Err(ChunkCoordOutOfRange)
            } else {
                self.$set_axis_unchecked($axis);
                Ok(())
            }
        }

        pub fn $set_axis_unchecked(&mut self, $axis: u8) {
            self.0 = (self.0 & !(($max as u16) << $offset)) | (($axis as u16) << $offset);
        }

        pub fn $axis(&self) -> u8 {
            (self.0 >> $offset & $max as u16) as u8
        }
    };
}

#[derive(Debug, thiserror::Error)]
#[error("The chunk coordinate was out of range. x: [0,31], y: [0,63], z: [0, 31]")]
pub struct ChunkCoordOutOfRange;

#[derive(Eq, PartialEq, Default, Copy, Clone)]
pub struct ChunkPos(pub u16);

impl fmt::Debug for ChunkPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ChunkPos(0x{:016b})", self.0)
    }
}

impl ChunkPos {
    pub fn new(x: u8, y: u8, z: u8) -> Result<Self, ChunkCoordOutOfRange> {
        if x >= CHUNK_SIZE.x || y >= CHUNK_SIZE.y || z >= CHUNK_SIZE.z {
            return Err(ChunkCoordOutOfRange);
        }

        Ok(Self::new_unchecked(x, y, z))
    }

    pub fn new_unchecked(x: u8, y: u8, z: u8) -> Self {
        let mut pos = x as u16;
        pos |= (y as u16) << 5;
        pos |= (z as u16) << 11;

        Self(pos)
    }

    impl_getters_setters!(x, set_x, set_x_unchecked, 0, CHUNK_SIZE.x - 1);
    impl_getters_setters!(y, set_y, set_y_unchecked, 5, CHUNK_SIZE.y - 1);
    impl_getters_setters!(z, set_z, set_z_unchecked, 11, CHUNK_SIZE.z - 1);

    pub fn center() -> Self {
        Self::try_from(CHUNK_SIZE / 2).expect("must be valid position")
    }
}

impl TryFrom<&TVec3<u8>> for ChunkPos {
    type Error = ChunkCoordOutOfRange;

    fn try_from(pos: &TVec3<u8>) -> Result<Self, Self::Error> {
        Self::new(pos.x, pos.y, pos.z)
    }
}

impl TryFrom<TVec3<u8>> for ChunkPos {
    type Error = ChunkCoordOutOfRange;

    fn try_from(pos: TVec3<u8>) -> Result<Self, Self::Error> {
        (&pos).try_into()
    }
}

impl From<&ChunkPos> for TVec3<u8> {
    fn from(chunk_pos: &ChunkPos) -> Self {
        Self::new(chunk_pos.x(), chunk_pos.y(), chunk_pos.z())
    }
}

impl From<ChunkPos> for TVec3<u8> {
    fn from(chunk_pos: ChunkPos) -> Self {
        (&chunk_pos).into()
    }
}

impl From<&WorldLocation> for ChunkPos { // TODO: maybe convert from WorldLocation -> BlockLocation -> ChunkPos?
    fn from(value: &WorldLocation) -> Self {
        use num_traits::cast::ToPrimitive;
        
        value
            .0
            .map_with_location(|r, _, n| { // TODO: add tests for this function
                n.floor()
                    .rem_euclid(CHUNK_SIZE[r] as f32)
                    .to_u8()
                    .expect("rem_euclid should be within range")
            })
            .try_into()
            .expect("values in range")
    }
}

impl From<&BlockLocation> for ChunkPos {
    fn from(loc: &BlockLocation) -> Self { // TODO: add tests for this function
        loc
            .0
            .map_with_location(|r, _, n| n.rem_euclid(CHUNK_SIZE[r] as _) as _)
            .try_into()
            .expect("values in range")
    }
}

impl From<WorldLocation> for ChunkPos {
    fn from(value: WorldLocation) -> Self {
        (&value).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_unchecked() {
        for x in 0..CHUNK_SIZE.x {
            for y in 0..CHUNK_SIZE.y {
                for z in 0..CHUNK_SIZE.z {
                    let pos = ChunkPos::new_unchecked(x, y, z);

                    let mut pos2 = ChunkPos(0xffff);

                    pos2.set_x_unchecked(x);
                    pos2.set_y_unchecked(y);
                    pos2.set_z_unchecked(z);

                    assert_eq!(pos, pos2);

                    assert_eq!(pos.x(), x);
                    assert_eq!(pos.y(), y);
                    assert_eq!(pos.z(), z);
                }
            }
        }
    }

    #[test]
    pub fn test_checked() {
        assert!(ChunkPos::new(31, 63, 31).is_ok());
        assert!(ChunkPos::new(32, 63, 31).is_err());
        assert!(ChunkPos::new(31, 64, 31).is_err());
        assert!(ChunkPos::new(31, 63, 32).is_err());

        let mut pos = ChunkPos(0xffff);

        assert!(pos.set_x(31).is_ok());
        assert!(pos.set_y(63).is_ok());
        assert!(pos.set_z(31).is_ok());

        assert!(pos.set_x(32).is_err());
        assert!(pos.set_y(64).is_err());
        assert!(pos.set_z(32).is_err());
    }
}