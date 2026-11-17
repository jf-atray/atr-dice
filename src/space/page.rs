use super::{Price, Pip, SLOTS, Slot};

// an index into links. minted by find an erange, nobody else.
#[derive(Clone, Copy)]
pub struct EdgeIdx(pub(crate) usize);

//arena alloc
pub struct Page {
    taken: u128,
    starts: [u16; SLOTS + 1],
    links: Vec<(Pip, Price)>,
}

impl Page {
    pub const fn new() -> Self {
        Self {
            taken: 0,
            starts: [0; SLOTS + 1],
            links: Vec::new(),
        }
    }
    pub fn with_link_capacity(n: usize) -> Self {
        Self {
            taken: 0,
            starts: [0; SLOTS + 1],
            links: Vec::with_capacity(n),
        }
    }
    // first free slot flips to taken. none when the page is full.
    pub fn make(&mut self) -> Option<Slot> {
        let free = (!self.taken).trailing_zeros();
        if free as usize >= SLOTS {
            return None;
        }
        self.taken |= 1 << free;
        Some(Slot(u8::try_from(free).ok()?))
    }
    pub const fn has(&self, s: Slot) -> bool {
        self.taken >> s.0 & 1 == 1
    }
    // s is under 128 so the offset reads cant miss.
    pub fn erange(&self, s: Slot) -> impl Iterator<Item = EdgeIdx> {
        let (lo, hi) = (self.starts[s.ix()], self.starts[s.ix() + 1]);
        (usize::from(lo)..usize::from(hi)).map(EdgeIdx)
    }
    // (dst, price) pairs in the slots range.
    pub fn edges(&self, s: Slot) -> impl Iterator<Item = (Pip, Price)> + '_ {
        self.erange(s).map(|i| unsafe { self.edge_unchecked(i) })
    }
    // index of s's link to b, if it exists.
    pub fn find(&self, s: Slot, b: Pip) -> Option<EdgeIdx> {
        self.erange(s).find(|&i| self.links[i.0].0 == b)
    }
    // offsets r u16. the pool holds 65535 half links at most.
    pub const fn has_room(&self) -> bool {
        self.links.len() < u16::MAX as usize
    }
    // sorted offsets, last one is the pool len, all u16 sized.
    pub fn invariant(&self) -> bool {
        self.starts[0] == 0
            && self.starts.windows(2).all(|w| w[0] <= w[1])
            && self.starts[SLOTS] as usize == self.links.len()
    }
    // EdgeIdx only comes from find or erange so its always inside links.
    pub fn edge(&self, i: EdgeIdx) -> (Pip, Price) {
        self.links[i.0]
    }
    // EdgeIdx only comes from find or erange so its always inside links.
    pub unsafe fn edge_unchecked(&self, i: EdgeIdx) -> (Pip, Price) {
        unsafe { *self.links.get_unchecked(i.0) }
    }
    // a slots half link count.
    pub fn ecount(&self, s: Slot) -> usize {
        (self.starts[s.ix() + 1] - self.starts[s.ix()]) as usize
    }
    // k comes from 0..ecount so its always inside links.
    pub fn edge_at(&self, s: Slot, k: usize) -> (Pip, Price) {
        self.links[self.starts[s.ix()] as usize + k]
    }
    // k comes from 0..ecount so its always inside links.
    pub unsafe fn edge_at_unchecked(&self, s: Slot, k: usize) -> (Pip, Price) {
        unsafe { *self.links.get_unchecked(self.starts[s.ix()] as usize + k) }
    }
    // one side of a link. the other side is in bs pool.
    pub fn half_link(&mut self, a: Slot, b: Pip, c: Price) {
        let at = self.starts[a.ix()] as usize;
        self.links.insert(at, (b, c));
        for o in &mut self.starts[a.ix() + 1..] {
            *o += 1;
        }
    }
    pub fn half_cut(&mut self, a: Slot, b: Pip) -> Option<Price> {
        let i = self.find(a, b)?;
        let (_, c) = self.links.remove(i.0);
        for o in &mut self.starts[a.ix() + 1..] {
            *o -= 1;
        }
        Some(c)
    }
    pub fn half_set(&mut self, a: Slot, b: Pip, c: Price) -> Option<()> {
        let i = self.find(a, b)?;
        self.links[i.0].1 = c;
        Some(())
    }
}

impl Default for Page {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::space::pip;

    #[test]
    fn make_fills_then_drys_up() {
        let mut p = Page::new();
        for i in 0..SLOTS {
            assert_eq!(p.make().unwrap().ix(), i);
        }
        assert!(p.make().is_none());
    }

    #[test]
    fn half_link_inserts_at_the_slots_front() {
        let mut p = Page::new();
        let a = p.make().unwrap();
        let b = p.make().unwrap();
        p.half_link(a, pip(0, b), 4);
        p.half_link(b, pip(0, a), 4);
        p.half_link(a, pip(3, a), 9);
        assert!(p.invariant());
        assert_eq!(p.ecount(a), 2);
        assert_eq!(p.ecount(b), 1);
        assert_eq!(p.find(a, pip(0, b)).map(|i| p.edge(i).1), Some(4));
    }

    #[test]
    fn half_cut_shifts_the_pool_down() {
        let mut p = Page::new();
        let a = p.make().unwrap();
        let b = p.make().unwrap();
        let c = p.make().unwrap();
        p.half_link(a, pip(0, b), 1);
        p.half_link(a, pip(0, c), 2);
        p.half_link(b, pip(0, a), 1);
        assert_eq!(p.half_cut(a, pip(0, b)), Some(1));
        assert!(p.invariant());
        assert!(p.find(a, pip(0, b)).is_none());
        assert_eq!(p.find(a, pip(0, c)).map(|i| p.edge(i).1), Some(2));
        // bs half was never as problem.
        assert_eq!(p.ecount(b), 1);
    }

    #[test]
    fn edge_at_matches_edges() {
        let mut p = Page::new();
        let a = p.make().unwrap();
        let b = p.make().unwrap();
        let c = p.make().unwrap();
        p.half_link(a, pip(0, b), 5);
        p.half_link(a, pip(0, c), 6);
        let via_iter: Vec<_> = p.edges(a).collect();
        let via_idx: Vec<_> = (0..p.ecount(a)).map(|k| p.edge_at(a, k)).collect();
        assert_eq!(via_iter, via_idx);
    }
}
