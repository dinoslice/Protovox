use shipyard::Unique;

#[derive(Debug, Unique, Eq, PartialEq)]
pub enum BlockBarDisplay {
    Expanded {
        selected: u8,
    },
    Minimized {
        start: u8,
        selected: u8
    },
}

impl BlockBarDisplay {
    const MINIMIZED_CT: u8 = 3;
    const SECTIONS: u8 = 3;
    const EXPANDED_CT: u8 = Self::MINIMIZED_CT * Self::SECTIONS;

    pub fn toggle(&mut self) {
        match self {
            BlockBarDisplay::Expanded { mut selected } => {
                *self = Self::Minimized {
                    start: selected - (selected % Self::MINIMIZED_CT),
                    selected,
                }
            }
            BlockBarDisplay::Minimized { mut selected, .. } => {
                *self = Self::Expanded { selected }
            }
        }
    }

    pub fn visible_spaces(&self) -> u8 {
        match self {
            BlockBarDisplay::Expanded { .. } => Self::EXPANDED_CT,
            BlockBarDisplay::Minimized { .. } => Self::MINIMIZED_CT,
        }
    }

    pub fn start(&self) -> u8 {
        match self {
            BlockBarDisplay::Expanded { .. } => 0,
            BlockBarDisplay::Minimized { start, .. } => *start,
        }
    }

    pub fn selected(&self) -> u8 {
        match self {
            BlockBarDisplay::Expanded { selected } => *selected,
            BlockBarDisplay::Minimized { selected, .. } => *selected,
        }
    }

    pub fn visible(&self) -> impl Iterator<Item = (u8, bool)> {
        let start = self.start();
        let count = self.visible_spaces();
        let selected = self.selected();

        (start..start + count)
            .rev()
            .map(move |n| (n, n == selected))
    }

    pub fn scroll(&mut self, amt: i32) {
        let section_step = (self.selected() - self.start()) % Self::EXPANDED_CT;

        let new_step = (section_step as i32 + amt).rem_euclid(self.visible_spaces() as _) as u8;

        let new_selected = (new_step + self.start()).rem_euclid(Self::EXPANDED_CT);

        match self {
            BlockBarDisplay::Expanded { selected } => *selected = new_selected,
            BlockBarDisplay::Minimized { selected, .. } => *selected = new_selected,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll() {
        let mut inv = BlockBarDisplay::Minimized { start: 0, selected: 0 };

        inv.scroll(1);

        assert_eq!(inv, BlockBarDisplay::Minimized { start: 0, selected: 1 });

        inv.scroll(1);

        assert_eq!(inv, BlockBarDisplay::Minimized { start: 0, selected: 2 });

        inv.scroll(1);

        assert_eq!(inv, BlockBarDisplay::Minimized { start: 0, selected: 0 });
    }
}