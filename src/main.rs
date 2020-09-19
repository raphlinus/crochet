//! A test binary, should move to example

use druid::{AppLauncher, PlatformError, Widget, WindowDesc};

use crochet::{AppHolder, Button, Cx, DruidAppData, Label, Row};

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
        Label::new(format!("current count: {}", self.count)).build(cx);
        if Button::new("Increment").build(cx) {
            self.count += 1;
        }
        Row::new().build(cx, |cx| {
            Button::new("A button").build(cx);
            Button::new("Another button").build(cx);
        });
        if self.count > 3 && self.count < 6 {
            Label::new("You did it!").build(cx);
        }
    }
}

fn ui_builder() -> impl Widget<DruidAppData> {
    let mut app_logic = MyAppLogic::default();

    AppHolder::new(move |cx| app_logic.run(cx))
}
