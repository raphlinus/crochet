//! The classic counter example but split up into component

use druid::{AppLauncher, PlatformError, Widget, WindowDesc};

use crochet::{component, AppHolder, Button, Column, Cx, DruidAppData, Label, Row};

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder);
    let data = Default::default();
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
}

#[derive(Default)]
struct MyAppLogic {
    count1: usize,
    count2: usize,
}

impl MyAppLogic {
    fn run(&mut self, cx: &mut Cx) {
        Row::new().build(cx, |cx| {
            counter(&mut self.count1, cx);
            counter(&mut self.count2, cx);
        });
    }
}

#[component]
fn counter(count: &mut usize, cx: &mut Cx) {
    cx.if_changed(*count, |cx| {
        Column::new().build(cx, |cx| {
            Label::new(format!("current count: {}", *count)).build(cx);
            if Button::new("Increment").build(cx) {
                *count += 1;
            }
            if *count > 3 && *count < 6 {
                Label::new("You did it!").build(cx);
            }
        });
    });
}

fn ui_builder() -> impl Widget<DruidAppData> {
    let mut app_logic = MyAppLogic::default();

    AppHolder::new(move |cx| app_logic.run(cx))
}
