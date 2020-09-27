//! A button which can move through a list of labels.

use druid::{AppLauncher, PlatformError, Widget, WindowDesc};

use crochet::{AppHolder, Button, Cx, DruidAppData, Id, Label, List, ListData, Row};

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder);
    let data = Default::default();
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
}

#[derive(Default)]
struct MyAppLogic {
    data: ListData<String>,
    list_view: List,
}

impl MyAppLogic {
    fn run(&mut self, cx: &mut Cx) {
        let mut swap_upwards_id = None;
        let mut swap_downwards_id = None;
        self.list_view
            .run(cx, &self.data, |cx, _is_selected, id: Id, item| {
                Row::new().build(cx, |cx| {
                    if Button::new("Up").build(cx) {
                        swap_upwards_id = Some(id);
                    }
                    Label::new(item.clone()).build(cx);
                    if Button::new("Down").build(cx) {
                        swap_downwards_id = Some(id);
                    }
                });
            });
        if let Some(id) = swap_upwards_id {
            let this_ix = self.data.find_id(id).unwrap();
            if this_ix > 0 {
                self.data.swap(this_ix - 1, this_ix);
            }
        }
        if let Some(id) = swap_downwards_id {
            let this_ix = self.data.find_id(id).unwrap();
            if this_ix < self.data.len() - 1 {
                self.data.swap(this_ix, this_ix + 1);
            }
        }
    }
}

fn ui_builder() -> impl Widget<DruidAppData> {
    let mut app_logic = MyAppLogic::default();
    app_logic.data.push("Alpha".into());
    app_logic.data.push("Beta".into());
    app_logic.data.push("Gamma".into());
    app_logic.data.push("Delta".into());

    AppHolder::new(move |cx| app_logic.run(cx))
}
