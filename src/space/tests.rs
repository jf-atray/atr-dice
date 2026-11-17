use super::domain::Bad;
use super::{Domain, Page, Reach, TitlePage, lanes, page, pip, slot};

fn dom(pages: u32) -> Domain {
    let mut w = Domain::default();
    for _ in 0..pages {
        w.pages.push(Page::new());
        w.titles.push(TitlePage::new());
    }
    w
}

#[test]
fn pip_packs_page_over_lane() {
    for pg in [0, 1, 5, 255] {
        for s in lanes() {
            let e = pip(pg, s);
            assert_eq!(page(e), pg);
            assert_eq!(slot(e), s);
        }
    }
}

#[test]
fn link_stores_both_halves() {
    let mut w = dom(1);
    let a = w.make(0).unwrap();
    let b = w.make(0).unwrap();
    w.link(a, b, 5).unwrap();
    assert_eq!(w.price(a, b), Some(5));
    assert_eq!(w.price(b, a), Some(5));
    assert!(w.pages[0].invariant());
}

#[test]
fn cut_removes_both_halves() {
    let mut w = dom(1);
    let a = w.make(0).unwrap();
    let b = w.make(0).unwrap();
    w.link(a, b, 5).unwrap();
    assert_eq!(w.cut(a, b), Ok(5));
    assert!(w.price(a, b).is_none());
    assert!(w.price(b, a).is_none());
    assert_eq!(w.cut(a, b), Err(Bad::NoLink));
    assert!(w.pages[0].invariant());
}

#[test]
fn reprice_rewrites_both_halves() {
    let mut w = dom(1);
    let a = w.make(0).unwrap();
    let b = w.make(0).unwrap();
    w.link(a, b, 5).unwrap();
    w.reprice(a, b, 9).unwrap();
    assert_eq!(w.price(a, b), Some(9));
    assert_eq!(w.price(b, a), Some(9));
}

#[test]
fn sever_sums_or_dies() {
    let mut w = dom(1);
    let a = w.make(0).unwrap();
    let b = w.make(0).unwrap();
    let mid = w.make(0).unwrap();
    w.link(a, b, 10).unwrap();
    assert_eq!(w.sever(a, b, mid, 4, 5), Err(Bad::Math));
    w.sever(a, b, mid, 4, 6).unwrap();
    assert!(w.price(a, b).is_none());
    assert_eq!(w.price(a, mid), Some(4));
    assert_eq!(w.price(mid, b), Some(6));
    assert!(w.pages[0].invariant());
}

#[test]
fn budget_walks_local_then_doors() {
    let mut w = dom(2);
    let a = w.make(0).unwrap();
    let b = w.make(0).unwrap();
    let far = w.make(1).unwrap();
    w.link(a, b, 7).unwrap();
    w.link(b, far, 3).unwrap();
    let me = w.make(0).unwrap();
    w.go(me, a).unwrap();
    let got: Vec<_> = w.within_budget(me, 10).collect();
    assert_eq!(
        got,
        vec![(a, 0, Reach::Node), (b, 7, Reach::Node), (far, 10, Reach::Door)]
    );
}

#[test]
fn budget_stops_at_the_cap() {
    let mut w = dom(1);
    let a = w.make(0).unwrap();
    let b = w.make(0).unwrap();
    w.link(a, b, 7).unwrap();
    let me = w.make(0).unwrap();
    w.go(me, a).unwrap();
    let got: Vec<_> = w.within_budget(me, 5).collect();
    assert_eq!(got, vec![(a, 0, Reach::Node)]);
}

#[test]
fn title_falls_back_to_the_id() {
    let mut w = dom(1);
    let a = w.make(0).unwrap();
    assert_eq!(w.title(a), format!("#{a}"));
    w.titles(a, "quay").unwrap();
    assert_eq!(w.title(a), "quay");
}
