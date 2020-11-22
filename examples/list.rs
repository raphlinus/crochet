//! A simple list view example.

use druid::{AppLauncher, PlatformError, Widget, WindowDesc};

use crochet::{AppHolder, Button, Column, Cx, DruidAppData, Id, Label, List, ListData, Row};

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
    counter: usize,
}

impl MyAppLogic {
    fn run(&mut self, cx: &mut Cx) {
        Column::new().build(cx, |cx| {
            Row::new().build(cx, |cx| {
                if Button::new("Create").build(cx) {
                    self.data.push(format!("item {}", self.counter));
                    self.counter += 1;
                }
                if Button::new("Delete").build(cx) {
                    if let Some(id) = self.list_view.selected() {
                        if let Some(ix) = self.data.find_id(id) {
                            self.data.remove_at_ix(ix);
                        }
                    }
                }
                if Button::new("Update").build(cx) {
                    if let Some(id) = self.list_view.selected() {
                        if let Some(ix) = self.data.find_id(id) {
                            self.data.set_at_ix(ix, format!("update {}", self.counter));
                            self.counter += 1;
                        }
                    }
                }
            });
            let mut new_sel = None;
            self.list_view
                .run(cx, &self.data, |cx, is_selected, id: Id, item| {
                    Row::new().build(cx, |cx| {
                        if Button::new("Select").build(cx) {
                            new_sel = Some(id);
                        }
                        let sel_str = if is_selected { "[*]" } else { "[ ]" };
                        Label::new(format!("{} {}", sel_str, item)).build(cx);
                    });
                });
            if let Some(id) = new_sel {
                self.list_view.select(id);
            }
        });
    }
}

fn ui_builder() -> impl Widget<DruidAppData> {
    let mut app_logic = MyAppLogic::default();

    AppHolder::new(move |cx| app_logic.run(cx))
}
