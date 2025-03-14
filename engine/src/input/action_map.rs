use strum::EnumCount;
use tinybitset::TinyBitSet;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

#[derive(Debug, Default, Clone)]
pub struct ActionMap(TinyBitSet<u8, { Action::COUNT.div_ceil(u8::BITS as _) }>);

#[repr(u8)]
#[derive(Copy, Clone, Debug, EnumCount)]
pub enum Action {
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    Jump,
    Sneak,
    PlaceBlock,
    BreakBlock,
    ToggleGamemode,
    ToggleFullscreen,
    ToggleBlockBar,

    BlockBar1,
    BlockBar2,
    BlockBar3,
    BlockBar4,
    BlockBar5,
    BlockBar6,
    BlockBar7,
    BlockBar8,
    BlockBar9,
}

impl Action {
    pub const BLOCK_BAR: [Self; 9] = [
        Self::BlockBar1,
        Self::BlockBar2,
        Self::BlockBar3,
        Self::BlockBar4,
        Self::BlockBar5,
        Self::BlockBar6,
        Self::BlockBar7,
        Self::BlockBar8,
        Self::BlockBar9,
    ];
}

impl ActionMap {
    pub fn process_input(&mut self, input: impl TryInto<Action>, is_pressed: bool) -> bool {
        match input.try_into() {
            Ok(action) => {
                self.set_action(action, is_pressed);
                true
            }
            Err(_) => false
        }
    }

    pub fn set_action(&mut self, action: Action, is_pressed: bool) {
        self.0.assign(action as usize, is_pressed);
    }

    pub fn get_action(&self, action: Action) -> bool {
        self.0[action as usize]
    }

    pub fn get_axis(&self, positive: Action, negative: Action) -> i8 {
        self.get_action(positive) as i8 - self.get_action(negative) as i8
    }

    pub fn reset_all(&mut self) {
        *self = Default::default();
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Could not convert to action")]
pub struct UnmappedAction;

impl TryFrom<&KeyCode> for Action {
    type Error = UnmappedAction;

    fn try_from(key: &KeyCode) -> Result<Self, Self::Error> {
        use KeyCode as KC;

        match key {
            KC::KeyW => Ok(Self::MoveForward),
            KC::KeyS => Ok(Self::MoveBackward),
            KC::KeyA => Ok(Self::MoveLeft),
            KC::KeyD => Ok(Self::MoveRight),
            KC::Space => Ok(Self::Jump),
            KC::ShiftLeft => Ok(Self::Sneak),
            KC::F4 => Ok(Self::ToggleGamemode),
            KC::F11 => Ok(Self::ToggleFullscreen),
            KC::KeyI => Ok(Self::ToggleBlockBar),

            KC::Digit1 => Ok(Self::BlockBar1),
            KC::Digit2 => Ok(Self::BlockBar2),
            KC::Digit3 => Ok(Self::BlockBar3),
            KC::Digit4 => Ok(Self::BlockBar4),
            KC::Digit5 => Ok(Self::BlockBar5),
            KC::Digit6 => Ok(Self::BlockBar6),
            KC::Digit7 => Ok(Self::BlockBar7),
            KC::Digit8 => Ok(Self::BlockBar8),
            KC::Digit9 => Ok(Self::BlockBar9),
            _ => Err(UnmappedAction),
        }
    }
}

impl TryFrom<&MouseButton> for Action {
    type Error = UnmappedAction;

    fn try_from(button: &MouseButton) -> Result<Self, Self::Error> {
        use MouseButton as MB;
        
        match button {
            MB::Left => Ok(Self::BreakBlock),
            MB::Right => Ok(Self::PlaceBlock),
            _ => Err(UnmappedAction),
        }
    }
}