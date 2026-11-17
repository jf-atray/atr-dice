#![allow(dead_code)]

mod column;
mod domain;
mod page;
#[cfg(test)]
mod tests;
mod title;

pub use column::Column;
pub use domain::{Domain, Reach};
pub use page::Page;
pub use title::TitlePage;

pub type Pip = u32;
pub type Price = u32;

pub const SLOTS: usize = 128;

// lane id bits an the mask slot() uses.
const LANES: u32 = SLOTS.trailing_zeros();
const SLOT_MASK: Pip = (1 << LANES) - 1;

// an index into the lanes. minted by slot(), make(), an lanes().
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot(pub(crate) u8);

impl Slot {
    pub fn ix(self) -> usize {
        usize::from(self.0)
    }
}

pub fn lanes() -> impl Iterator<Item = Slot> {
    (0..u8::try_from(SLOTS).unwrap()).map(Slot)
}

// pip = page in the high bits, lane in the low ones.
pub const fn page(e: Pip) -> u32 {
    e >> LANES
}
pub const fn slot(e: Pip) -> Slot {
    Slot((e & SLOT_MASK) as u8)
}
pub fn pip(pg: u32, s: Slot) -> Pip {
    pg << LANES | u32::from(s.0)
}
