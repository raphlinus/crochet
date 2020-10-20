//! The classic counter example.

use druid::{AppLauncher, PlatformError, Widget, WindowDesc};

use crochet::{AppHolder, Column, Cx, DruidAppData, Label, TextBox};

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder);
    let data = Default::default();
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
}

fn ui_builder() -> impl Widget<DruidAppData> {
    let mut initial_state = HelloState {
        name: "World".to_string(),
    };

    AppHolder::new(move |cx| initial_state.run(cx))
}

struct HelloState {
    name: String,
}

impl HelloState {
    fn name_label(&self) -> String {
        if self.name.is_empty() {
            "Hello anybody!?".to_string()
        } else {
            format!("Hello {}!", self.name)
        }
    }

    fn run(&mut self, cx: &mut Cx) {
        Column::new().build(cx, |cx| {
            Label::new(self.name_label()).build(cx);
            if let Some(name) = TextBox::new(&self.name).build(cx) {
                self.name = name;
            }
        });
    }
}
