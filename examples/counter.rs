//! The classic counter example.

use druid::{AppLauncher, PlatformError, Widget, WindowDesc};

use crochet::{AppHolder, Button, Clicked, Column, Cx, DruidAppData, Label, Padding};

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
        cx.if_changed(self.count, |cx| {
            println!("traversing into if_changed block");
            Column::new().build(cx, |cx| {
                Label::new(format!("current count: {}", self.count)).build(cx);
                if Button::new("Increment").build(cx) {
                    self.count += 1;
                }
                if self.count > 3 && self.count < 6 {
                    Padding::new().top(10.0).build(cx, |cx| {
                        let clicked = Clicked::new().build(cx, |cx| {
                            Label::new("You did it! Now click here!").build(cx);
                        });
                        if clicked {
                            println!("It's over 9000!");
                            self.count += 9000;
                        }
                    });
                }
            });
        });
    }
}

fn ui_builder() -> impl Widget<DruidAppData> {
    let mut app_logic = MyAppLogic::default();

    AppHolder::new(move |cx| app_logic.run(cx))
}
