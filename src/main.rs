//! A test binary, should move to example

use crochet::{Cx, Tree};

fn run(cx: &mut Cx) {
    cx.begin("hello");
    cx.end();
}

fn main() {
    let tree = Tree::new("root");
    println!("{:?}", tree);
    let mut cx = Cx::new(tree);
    run(&mut cx);
    let tree = cx.into_tree();
    println!("{:?}", tree);
}
