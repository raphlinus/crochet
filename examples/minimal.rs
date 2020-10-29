//! A minimal example.

use crochet::{AppHolder, Cx, DruidAppData};
use druid::{AppLauncher, PlatformError, Widget, WindowDesc};

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder);
    let data = Default::default();
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
}

fn ui_builder() -> impl Widget<DruidAppData> {
    AppHolder::new(run)
}

fn run(cx: &mut Cx) {
    crochet::Label::new("Hello, world!").build(cx);
}
