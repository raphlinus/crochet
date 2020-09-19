//! A test binary, should move to example

use druid::{AppLauncher, PlatformError, Widget, WindowDesc};

use crochet::{AppHolder, Cx, MutIterItem, Mutation, MutationIter, Tree};

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

fn debug_report(tree: &Tree, mutation: &Mutation) {
    tree.dump();
    println!("{:?}", mutation);
    let mut_iter = MutationIter::new(tree, mutation);
    debug_print_mutation(mut_iter, 0);
}

#[allow(unused)]
fn crochet_toy() {
    let mut tree = Tree::default();

    let mut cx = Cx::new(&tree);
    run(&mut cx, 1, 1);
    let mutation = cx.into_mutation();
    debug_report(&tree, &mutation);
    tree.mutate(mutation);

    let mut cx = Cx::new(&tree);
    run(&mut cx, 2, 1);
    let mutation = cx.into_mutation();
    debug_report(&tree, &mutation);
    tree.mutate(mutation);
    tree.dump();
}

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder);
    let data = ();
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
}

struct MyAppLogic;

impl MyAppLogic {
    fn run(&mut self, cx: &mut Cx) {
        cx.leaf("button: Hello");
        cx.begin("row");
        cx.leaf("button: 1");
        cx.leaf("button: 2");
        cx.end();
        cx.leaf("button: World");
    }
}

fn ui_builder() -> impl Widget<()> {
    let mut app_logic = MyAppLogic;

    AppHolder::new(move |cx| app_logic.run(cx))
}
