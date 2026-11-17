use super::{Page, Pip, Price, SLOTS, Slot, TitlePage, lanes, page, pip, slot};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reach {
    Node,
    Door,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bad {
    NoPage,
    Full,
    Free(Pip),
    NoLink,
    Dup,
    Same,
    Math,
}

#[derive(Default)]
pub struct Domain {
    pub pages: Vec<Page>,
    pub titles: Vec<TitlePage>,
    movers: Vec<(Pip, Pip)>, // movers r rare. where a thing stands.
}

impl Domain {
    fn check(&self, e: Pip) -> Result<(), Bad> {
        let p = self.pages.get(page(e) as usize).ok_or(Bad::NoPage)?;
        if p.has(slot(e)) {
            Ok(())
        } else {
            Err(Bad::Free(e))
        }
    }
    pub fn make(&mut self, pg: u32) -> Result<Pip, Bad> {
        let p = self.pages.get_mut(pg as usize).ok_or(Bad::NoPage)?;
        let s = p.make().ok_or(Bad::Full)?;
        Ok(pip(pg, s))
    }
    pub fn titles(&mut self, e: Pip, text: impl Into<String>) -> Result<(), Bad> {
        self.check(e)?;
        let t = self.titles.get_mut(page(e) as usize).ok_or(Bad::NoPage)?;
        *t.at_mut(slot(e)) = Some(text.into());
        Ok(())
    }
    // where a thing counts as. movers r at their here, everythin else is its own node.
    fn move_or(&self, e: Pip) -> Pip {
        self.movers.iter().find(|&&(k, _)| k == e).map_or(e, |&(_, at)| at)
    }
    pub fn go(&mut self, e: Pip, node: Pip) -> Result<(), Bad> {
        self.check(e)?;
        self.check(node)?;
        if let Some((_, at)) = self.movers.iter_mut().find(|m| m.0 == e) {
            *at = node;
        } else {
            self.movers.push((e, node));
        }
        Ok(())
    }
    pub fn link(&mut self, a: Pip, b: Pip, c: Price) -> Result<(), Bad> {
        if a == b {
            return Err(Bad::Same);
        }
        self.check(a)?;
        self.check(b)?;
        if self.price(a, b).is_some() {
            return Err(Bad::Dup);
        }
        let (pa, pb) = (page(a) as usize, page(b) as usize);
        if !self.pages[pa].has_room() || !self.pages[pb].has_room() {
            return Err(Bad::Full);
        }
        self.pages[pa].half_link(slot(a), b, c);
        self.pages[pb].half_link(slot(b), a, c);
        Ok(())
    }
    // one hop if the edge exists.
    pub fn price(&self, a: Pip, b: Pip) -> Option<Price> {
        let p = self.pages.get(page(a) as usize)?;
        p.find(slot(a), b).map(|i| p.edge(i).1)
    }
    pub fn cut(&mut self, a: Pip, b: Pip) -> Result<Price, Bad> {
        self.check(a)?;
        self.check(b)?;
        let (pa, pb) = (page(a) as usize, page(b) as usize);
        let (sa, sb) = (slot(a), slot(b));
        if self.pages[pa].find(sa, b).is_none() || self.pages[pb].find(sb, a).is_none() {
            return Err(Bad::NoLink);
        }
        let c = self.pages[pa].half_cut(sa, b).unwrap();
        debug_assert_eq!(Some(c), self.pages[pb].half_cut(sb, a));
        Ok(c)
    }
    pub fn reprice(&mut self, a: Pip, b: Pip, c: Price) -> Result<(), Bad> {
        self.check(a)?;
        self.check(b)?;
        let (pa, pb) = (page(a) as usize, page(b) as usize);
        let (sa, sb) = (slot(a), slot(b));
        if self.pages[pa].find(sa, b).is_none() || self.pages[pb].find(sb, a).is_none() {
            return Err(Bad::NoLink);
        }
        self.pages[pa].half_set(sa, b, c);
        self.pages[pb].half_set(sb, a, c);
        Ok(())
    }
    // an edge splits at a real node. back an fore must total the old price.
    pub fn sever(&mut self, a: Pip, b: Pip, mid: Pip, back: Price, fore: Price) -> Result<(), Bad> {
        if mid == a || mid == b {
            return Err(Bad::Same);
        }
        self.check(a)?;
        self.check(b)?;
        self.check(mid)?;
        let c = self.price(a, b).ok_or(Bad::NoLink)?;
        if back.checked_add(fore) != Some(c) {
            return Err(Bad::Math);
        }
        if self.price(a, mid).is_some() || self.price(mid, b).is_some() {
            return Err(Bad::Dup);
        }
        if !self.pages[page(mid) as usize].has_room() {
            return Err(Bad::Full);
        }
        self.cut(a, b)?;
        self.link(a, mid, back)?;
        self.link(mid, b, fore)?;
        Ok(())
    }
    // a lazy dijkstra. nodes come out in price order, doors after.
    pub fn within_budget(&self, me: Pip, budget: Price) -> impl Iterator<Item = (Pip, Price, Reach)> {
        const SET: Price = 1 << (Price::BITS - 1);
        // top bit is settled, max is unreached. real prices stay under it.
        let start = self.move_or(me);
        let pg = page(start);
        let pg_ref = self.pages.get(pg as usize);
        let ok = pg_ref.is_some_and(|p| p.has(slot(start)));
        gen move {
            if !ok {
                return;
            }
            // ok was the page check. this unwrap cant miss.
            let p = pg_ref.unwrap();
            let mut dist = [Price::MAX; SLOTS];
            dist[slot(start).ix()] = 0;
            loop {
                let mut bi = usize::MAX;
                let mut bd = Price::MAX;
                for (i, &d) in dist.iter().enumerate() {
                    if d & SET == 0 && d < bd {
                        bd = d;
                        bi = i;
                    }
                }
                if bi == usize::MAX || bd > budget {
                    break;
                }
                // bi is always a lane so the u8 try cant miss.
                let bs = Slot(u8::try_from(bi).unwrap());
                for (to, c) in p.edges(bs) {
                    let d2 = bd.saturating_add(c);
                    let ds = slot(to).ix();
                    if page(to) == pg && d2 < dist[ds] & !SET {
                        dist[ds] = d2;
                    }
                }
                dist[bi] |= SET;
                yield (pip(pg, bs), bd, Reach::Node);
            }
            // second pass. settled slots only, cross page edges r the doors.
            for s in lanes() {
                let base = dist[s.ix()];
                if base & SET == 0 {
                    continue;
                }
                let base = base & !SET;
                for (to, c) in p.edges(s) {
                    if page(to) != pg {
                        let d2 = base.saturating_add(c);
                        if d2 <= budget {
                            yield (to, d2, Reach::Door);
                        }
                    }
                }
            }
        }
    }
    pub fn title(&self, e: Pip) -> String {
        self.titles
            .get(page(e) as usize)
            .and_then(|t| t.at(slot(e)).clone())
            .unwrap_or_else(|| format!("#{e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::space::{Page, TitlePage, pip};

    fn dom(pages: u32) -> Domain {
        let mut w = Domain::default();
        for _ in 0..pages {
            w.pages.push(Page::new());
            w.titles.push(TitlePage::new());
        }
        w
    }

    #[test]
    fn make_on_a_missing_page() {
        assert_eq!(Domain::default().make(0), Err(Bad::NoPage));
    }

    #[test]
    fn link_refuses_self_dup_and_dead() {
        let mut w = dom(1);
        let a = w.make(0).unwrap();
        let b = w.make(0).unwrap();
        assert_eq!(w.link(a, a, 1), Err(Bad::Same));
        w.link(a, b, 5).unwrap();
        assert_eq!(w.link(a, b, 5), Err(Bad::Dup));
        let dead = pip(0, slot(120));
        assert_eq!(w.link(a, dead, 1), Err(Bad::Free(dead)));
    }

    #[test]
    fn go_and_titles_want_taken_pips() {
        let mut w = dom(1);
        let a = w.make(0).unwrap();
        let free = pip(0, slot(9));
        assert_eq!(w.go(free, a), Err(Bad::Free(free)));
        assert_eq!(w.titles(free, "x"), Err(Bad::Free(free)));
    }

    #[test]
    fn a_mover_counts_as_its_node() {
        let mut w = dom(1);
        let a = w.make(0).unwrap();
        let me = w.make(0).unwrap();
        w.go(me, a).unwrap();
        assert_eq!(w.move_or(me), a);
    }
}
