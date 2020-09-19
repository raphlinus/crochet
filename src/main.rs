//! A test binary, should move to example

use druid::{AppLauncher, PlatformError, Widget, WindowDesc};

use crochet::{AppHolder, Cx, DruidAppData, MutIterItem, Mutation, MutationIter, Tree};

// Some random tree mutation testing functions, unused.

#[allow(unused)]
fn run(cx: &mut Cx, num_a: usize, num_b: usize) {
    cx.begin("hello");
    for i in 0..num_a {
        cx.leaf(format!("a{}", i));
    }
    for i in 0..num_b {
        cx.leaf(format!("b{}", i));
    }
    cx.end();
}

#[allow(unused)]
fn debug_print_mutation(mut_iter: MutationIter, level: usize) {
    for item in mut_iter {
        let indent = "  ".repeat(level);
        match item {
            MutIterItem::Skip(n) => println!("{}Skip {}", indent, n),
            MutIterItem::Delete(n) => println!("{}Delete {}", indent, n),
            MutIterItem::Insert(id, body, children) => {
                println!("{}Insert {:?}: {}", indent, id, body);
                debug_print_mutation(children, level + 1);
            }
            MutIterItem::Update(body, children) => {
                println!("{}Update {:?}", indent, body);
                debug_print_mutation(children, level + 1);
            }
        }
    }
}

#[allow(unused)]
fn debug_report(tree: &Tree, mutation: &Mutation) {
    tree.dump();
    println!("{:?}", mutation);
    let mut_iter = MutationIter::new(tree, mutation);
    debug_print_mutation(mut_iter, 0);
}

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder);
    let data = Default::default();
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
}

#[derive(Default)]
struct MyAppLogic {
    count: usize,
}

impl MyAppLogic {
    fn run(&mut self, cx: &mut Cx) {
        cx.leaf("button: Hello");
        cx.begin("row");
        if cx.button("1") {
            self.count += 1;
        }
        cx.leaf("button: 2");
        cx.end();
        cx.leaf("button: World");
        if self.count > 3 && self.count < 6 {
            cx.leaf("button: woot!");
        }
    }
}

fn ui_builder() -> impl Widget<DruidAppData> {
    let mut app_logic = MyAppLogic::default();

    AppHolder::new(move |cx| app_logic.run(cx))
}
