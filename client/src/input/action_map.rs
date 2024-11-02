use tinybitset::TinyBitSet;
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
            // TODO: use mouse input
            KC::KeyN => Some(Self::PlaceBlock),
            KC::KeyM => Some(Self::BreakBlock),
            _ => None,
        }
    }
}

impl ActionMap {
    pub fn process_input(&mut self, key: &KeyCode, is_pressed: bool) -> bool {
        match Action::mapped_from_key(key) {
            Some(action) => {
                self.set_action(action, is_pressed);
                true
            }
            None => false
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