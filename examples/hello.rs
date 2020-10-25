//! The classic counter example.

use druid::{AppLauncher, PlatformError, Widget, WindowDesc};

use crochet::{AppHolder, Checkbox, Column, Cx, DruidAppData, Label, Padding, TextBox};

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
        german: false,
    };

    AppHolder::new(move |cx| initial_state.run(cx))
}

struct HelloState {
    name: String,
    german: bool,
}

impl HelloState {
    fn greeting(&self) -> &str {
        if self.german {
            "Guten Tag"
        } else {
            "Hello"
        }
    }

    fn name_label(&self) -> String {
        if self.name.is_empty() {
            "Hello anybody!?".to_string()
        } else {
            format!("{} {}!", self.greeting(), self.name)
        }
    }

    fn run(&mut self, cx: &mut Cx) {
        Padding::from(5.0 * (self.name.len() as f64)).build(cx, |cx| {
            Column::new().build(cx, |cx| {
                Label::new(self.name_label()).build(cx);
                Padding::new().top(5.0).build(cx, |cx| {
                    if let Some(name) = TextBox::new(&self.name).build(cx) {
                        self.name = name;
                    }
                });
                Padding::new().top(5.0).build(cx, |cx| {
                    self.german = Checkbox::new("greet in german", self.german).build(cx);
                });
            });
        });
    }
}
