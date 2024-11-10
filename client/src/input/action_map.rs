use tinybitset::TinyBitSet;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

const N: usize = std::mem::size_of::<Action>() * u8::BITS as usize;

#[derive(Debug, Default)]
pub struct ActionMap(TinyBitSet<u8, N>); // TODO: replace this with own bitset

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
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
}

impl Action {
    pub fn mapped_from_key(key: &KeyCode) -> Option<Self> {
        use KeyCode as KC;

        match key {
            KC::KeyW => Some(Self::MoveForward),
            KC::KeyS => Some(Self::MoveBackward),
            KC::KeyA => Some(Self::MoveLeft),
            KC::KeyD => Some(Self::MoveRight),
            KC::Space => Some(Self::Jump),
            KC::ShiftLeft => Some(Self::Sneak),
            _ => None,
        }
    }
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