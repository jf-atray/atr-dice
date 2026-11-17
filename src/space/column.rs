use super::{SLOTS, Slot};
use std::array;

// a fixed row of cells keyed by lane.
pub struct Column<T, const N: usize> {
    pub cells: [T; N],
}

impl<T: Default, const N: usize> Column<T, N> {
    pub fn new() -> Self {
        Self {
            cells: array::from_fn(|_| T::default()),
        }
    }
}

impl<T: Default, const N: usize> Default for Column<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Column<T, SLOTS> {
    // slots r masked under SLOTS so a SLOTS column cant miss.
    pub fn at(&self, s: Slot) -> &T {
        &self.cells[s.ix()]
    }
    // slots r masked under SLOTS so a SLOTS column cant miss.
    pub fn at_mut(&mut self, s: Slot) -> &mut T {
        &mut self.cells[s.ix()]
    }
    // slots r masked under SLOTS so a SLOTS column cant miss.
    pub unsafe fn at_unchecked(&self, s: Slot) -> &T {
        unsafe { self.cells.get_unchecked(s.ix()) }
    }
    // slots r masked under SLOTS so a SLOTS column cant miss.
    pub unsafe fn at_mut_unchecked(&mut self, s: Slot) -> &mut T {
        unsafe { self.cells.get_unchecked_mut(s.ix()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::space::slot;

    #[test]
    fn cells_read_write_by_lane() {
        let mut c: Column<u32, SLOTS> = Column::new();
        let s = slot(3);
        *c.at_mut(s) = 42;
        assert_eq!(*c.at(s), 42);
        assert_eq!(unsafe { *c.at_unchecked(s) }, 42);
    }
}
