use std::fmt;

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
            self.0 = (self.0 & !($max << $offset)) | (($axis as u16) << $offset);
        }

        pub fn $axis(&self) -> u8 {
            ((self.0 >> $offset) & $max) as u8
        }
    };
}

pub struct ChunkCoordOutOfRange;

#[derive(Eq, PartialEq, Default)]
pub struct ChunkPos(pub u16);

impl fmt::Debug for ChunkPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ChunkPos(0x{:016b})", self.0)
    }
}

impl ChunkPos {
    pub fn new(x: u8, y: u8, z: u8) -> Result<Self, ChunkCoordOutOfRange> {
        if x > 31 || y > 63 || z > 31 {
            return Err(ChunkCoordOutOfRange);
        }

        Ok(Self::new_unchecked(x, y, z))
    }

    pub fn new_unchecked(x: u8, y: u8, z: u8) -> Self {
        let mut pos = 0;

        pos = (pos & !(0x1f << 0)) | ((x as u16) << 0);
        pos = (pos & !(0x3f << 5)) | ((y as u16) << 5);
        pos = (pos & !(0x1f << 11)) | ((z as u16) << 11);

        Self(pos)
    }

    impl_getters_setters!(x, set_x, set_x_unchecked, 0, 31);
    impl_getters_setters!(y, set_y, set_y_unchecked, 5, 63);
    impl_getters_setters!(z, set_z, set_z_unchecked, 11, 31);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_unchecked() {
        for x in 0..32 {
            for y in 0..64 {
                for z in 0..32 {
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