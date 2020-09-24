//! The classic counter example.

use druid::{AppLauncher, PlatformError, Widget, WindowDesc};

use crochet::{AppHolder, Button, Column, Cx, DruidAppData, Label};

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
        // Note: the if_changed block here is not really necessary, but it
        // helps test it out.
        if let Some(mut cx) = cx.changed(self.count) {
            println!("traversing into if_changed block");
            let mut cx = Column::new().build(&mut cx);
            Label::new(format!("current count: {}", self.count)).build(&mut cx);
            if Button::new("Increment").build(&mut cx) {
                self.count += 1;
            }
            if self.count > 3 && self.count < 6 {
                Label::new("You did it!").build(&mut cx);
            }
        }
    }
}

fn ui_builder() -> impl Widget<DruidAppData> {
    let mut app_logic = MyAppLogic::default();

    AppHolder::new(move |cx| app_logic.run(cx))
}
