//! A test binary, should move to example

use crochet::Cx;

fn run(cx: &mut Cx) {
    let count = cx.state(|| 0u32);
    println!("current count: {}", count);
    *count += 1;
}

fn main() {
    let mut cx = Cx::default();
    cx.foo();
    run(&mut cx);
    run(&mut cx);
}
