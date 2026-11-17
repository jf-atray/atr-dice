#![feature(gen_blocks)]

mod space;

use space::{Domain, Page, Reach, TitlePage};

fn main() {
    let mut w = Domain::default();
    w.pages.push(Page::new());
    w.titles.push(TitlePage::new());

    let room = w.make(0).unwrap();
    w.titles(room, "room").unwrap();
    let far = w.make(0).unwrap();
    w.titles(far, "far").unwrap();
    w.link(room, far, 7).unwrap();

    let me = w.make(0).unwrap();
    w.go(me, room).unwrap();

    for (node, price, kind) in w.within_budget(me, 60) {
        match kind {
            Reach::Node => println!("{price:>3}  {}", w.title(node)),
            Reach::Door => println!("  -> {} for {price}", w.title(node)),
        }
    }
}
