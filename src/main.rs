//! A test binary, should move to example

use crochet::{MutCursor, Tree};

fn run(cx: &mut MutCursor, num_a: usize, num_b: usize) {
    cx.begin("hello".to_string());
    for i in 0..num_a {
        cx.leaf(format!("a{}", i));
    }
    for i in 0..num_b {
        cx.leaf(format!("b{}", i));
    }
    cx.end();
}

fn main() {
    let mut tree = Tree::default();
    tree.dump();

    let mut cx = MutCursor::new(&tree);
    run(&mut cx, 1, 1);
    let mutation = cx.into_mutation();
    println!("{:?}", mutation);
    tree.mutate(mutation);
    tree.dump();

    let mut cx = MutCursor::new(&tree);
    run(&mut cx, 2, 1);
    let mutation = cx.into_mutation();
    println!("{:?}", mutation);
    tree.mutate(mutation);
    tree.dump();

}
