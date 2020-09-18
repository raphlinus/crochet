//! A test binary, should move to example

use crochet::{MutCursor, MutIterItem, Mutation, MutationIter, Tree};

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

fn debug_print_mutation(mut_iter: MutationIter, level: usize) {
    for item in mut_iter {
        let indent = "  ".repeat(level);
        match item {
            MutIterItem::Skip(n) => println!("{}Skip {}", indent, n),
            MutIterItem::Delete(n) => println!("{}Delete {}", indent, n),
            MutIterItem::Insert(body, children) => {
                println!("{}Insert {}", indent, body);
                debug_print_mutation(children, level + 1);
            }
            MutIterItem::Update(body, children) => {
                println!("{}Update {:?}", indent, body);
                debug_print_mutation(children, level + 1);
            }
        }
    }
}

fn debug_report(tree: &Tree, mutation: &Mutation) {
    tree.dump();
    println!("{:?}", mutation);
    let mut_iter = MutationIter::new(tree, mutation);
    debug_print_mutation(mut_iter, 0);
}

fn main() {
    let mut tree = Tree::default();

    let mut cx = MutCursor::new(&tree);
    run(&mut cx, 1, 1);
    let mutation = cx.into_mutation();
    debug_report(&tree, &mutation);
    tree.mutate(mutation);

    let mut cx = MutCursor::new(&tree);
    run(&mut cx, 2, 1);
    let mutation = cx.into_mutation();
    debug_report(&tree, &mutation);
    tree.mutate(mutation);
    tree.dump();
}
