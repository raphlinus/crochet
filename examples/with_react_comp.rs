use crochet::react_builder::{
    Button, ComponentCaller, ElementList, ElementTree, ElementTuple, Label, ReactApp,
};
use crochet::react_comp::{ButtonPressed, EventEnum};
use crochet::react_ext::VirtualDomBuilderExt;

#[allow(unused_imports)]
use crochet::{AppHolder, Cx, DruidAppData, Id, List, ListData, Row};
use druid::{AppLauncher, PlatformError, Widget, WindowDesc};

#[derive(Debug, Clone)]
struct ListItem {
    text: String,
    id: i32,
}

struct AppState {
    data: Vec<ListItem>,
    selected_row: Option<usize>,
    next_id: i32,
}

type RowEvent = EventEnum<ButtonPressed, (), (), ()>;
struct RowProps {
    list_item: ListItem,
    is_selected: bool,
}

fn list_row(state: &u16, props: RowProps) -> impl ElementTree<u16, Event = RowEvent> {
    let age = *state;
    ElementTuple(
        Button::new("Select").with_event(|state: &mut u16, _| {
            *state += 1;
        }),
        Label::new(if props.is_selected { " [*]" } else { " [ ]" }),
        Label::new(format!("{} - age={}", &props.list_item.text, age)),
        Label::new(format!("id={}", props.list_item.id)),
        Default::default(),
    )
}

type AppEvent = EventEnum<ButtonPressed, ButtonPressed, ButtonPressed, (usize, RowEvent)>;

fn some_component(state: &AppState, _props: ()) -> impl ElementTree<AppState, Event = AppEvent> {
    let button_create = Button::new("Create").with_event(|state: &mut AppState, _| {
        state.data.push(ListItem {
            text: "new item".to_string(),
            id: state.next_id,
        });
        state.next_id += 1;
    });
    let button_delete = Button::new("Delete").with_event(|state: &mut AppState, _| {
        if let Some(row) = state.selected_row {
            state.data.remove(row as usize);
            state.selected_row = None;
        }
    });
    let button_update = Button::new("Update").with_event(|state: &mut AppState, _| {
        if let Some(row) = state.selected_row {
            state.data[row as usize].text = "updated".to_string();
        }
    });

    let list_view_data = state
        .data
        .iter()
        .enumerate()
        .map(|(i, list_item)| {
            let row_props = RowProps {
                list_item: list_item.clone(),
                is_selected: state.selected_row == Some(i),
            };

            let comp_builder = ComponentCaller::prepare(list_row, row_props);

            (list_item.id.to_string(), comp_builder)
        })
        .collect();
    let list_view = ElementList {
        elements: list_view_data,
        _state: Default::default(),
    };

    ElementTuple(
        button_create,
        button_delete,
        button_update,
        list_view.with_event(|state: &mut AppState, event| {
            let i = event.0;
            state.selected_row = Some(i);
        }),
        Default::default(),
    )
}

fn ui_builder() -> impl Widget<DruidAppData> {
    let state = AppState {
        data: (0..8_i32)
            .map(|i| ListItem {
                text: "hello".to_string(),
                id: i,
            })
            .collect(),
        selected_row: None,
        next_id: 8,
    };

    let mut react_component = ReactApp::new(&some_component, state);

    AppHolder::new(move |cx| {
        react_component.run(cx);
    })
}

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder);
    let data = Default::default();
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
}
