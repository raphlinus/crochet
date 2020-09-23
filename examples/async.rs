//! A very simple async example.

use druid::{AppLauncher, PlatformError, Widget, WindowDesc};

use crochet::{AppHolder, Button, Cx, DruidAppData, Label};

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
        if self.count > 3 && self.count < 6 {
            Label::new("You did it!").build(cx);
        }
        cx.use_future(
            self.count,
            |&val| async move {
                async_std::task::sleep(std::time::Duration::from_secs(1)).await;
                val * 2
            },
            |cx, result| {
                let text = if let Some(val) = result {
                    format!("value: {}", val)
                } else {
                    "waiting...".into()
                };
                Label::new(text).build(cx);
            },
        )
    }
}

fn ui_builder() -> impl Widget<DruidAppData> {
    let mut app_logic = MyAppLogic::default();

    AppHolder::new(move |cx| app_logic.run(cx))
}
