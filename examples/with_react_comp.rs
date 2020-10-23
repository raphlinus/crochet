use crochet::react_comp::{VirtualDom, ButtonPressed, EventEnum};
use crochet::react_builder::{VirtualDomBuilder, ReactApp, ComponentTuple, ComponentList, VDomLabel, VDomButton, ComponentBuilder};
use crochet::react_ext::VirtualDomBuilderExt;

#[allow(unused_imports)]
use crochet::{AppHolder, Button, Cx, DruidAppData, Id, Label, List, ListData, Row};

use druid::{AppLauncher, PlatformError, Widget, WindowDesc};


#[derive(Debug, Clone)]
struct ListItem {
    text: String,
    id: i32,
}

struct AppState {
    data: Vec<ListItem>,
    selected_row: Option<i32>,
    next_id: i32,
}


type RowEvent = EventEnum<ButtonPressed, (), (), ()>;
struct RowProps<'a> {
    list_item: &'a ListItem,
    is_selected: bool,
}

fn list_row(_state: &u16, props: RowProps) -> impl VirtualDom<u16, Event = RowEvent>
{
    ComponentTuple(
        VDomButton::new("Select"),
        VDomLabel::new(if props.is_selected { "[*]" } else { "[ ]" }),
        VDomLabel::new(props.list_item.text.clone()),
        VDomLabel::new(props.list_item.id.to_string()),
        Default::default(),
    ).build()
}


type AppEvent = EventEnum<
    ButtonPressed,
    ButtonPressed,
    ButtonPressed,
    (i32, RowEvent),
>;

fn some_component(state: &AppState, _props: &()) -> impl VirtualDom<AppState, Event = AppEvent> {
    let button_create = VDomButton::new("Create").with_event(|state: &mut AppState, _| {
        state.data.push(ListItem { text: "new item".to_string(), id: state.next_id });
        state.next_id += 1;
    });
    let button_delete = VDomButton::new("Delete").with_event(|state: &mut AppState, _| {
        if let Some(row) = state.selected_row {
            state.data.remove(row as usize);
            state.selected_row = None;
        }
    });
    let button_update = VDomButton::new("Update").with_event(|state: &mut AppState, _| {
        if let Some(row) = state.selected_row {
            state.data[row as usize].text = "updated".to_string();
        }
    });

    let list_view_data = state.data.iter().enumerate().map(|(i, list_item)| {
        let row_props = RowProps {
            list_item: &list_item,
            is_selected: i as i32 == state.selected_row.unwrap_or(-1),
        };

        let comp_builder = ComponentBuilder::prepare(list_row, row_props);

        (list_item.id.to_string(), comp_builder)
    }).collect();
    let list_view = ComponentList {
        components: list_view_data, _state: Default::default()
    };

    ComponentTuple(
        button_create,
        button_delete,
        button_update,
        list_view.with_event(|state: &mut AppState, event| {
            let i = event.0;
            state.selected_row = Some(i);
        }),
        Default::default(),
    ).build()
}


fn ui_builder() -> impl Widget<DruidAppData> {
    let state = AppState {
        data: (0..8_i32).map(|i| ListItem { text: "hello".to_string(), id: i }).collect(),
        selected_row: None,
        next_id: 8,
    };

    let mut react_component = ReactApp::<AppState, (), _, _> {
        state,
        root_component: &some_component,
        prev_vdom: None,
        prev_vdom_state: None,
        _props: Default::default(),
    };

    AppHolder::new(move |cx| {
        react_component.run(cx, &mut (), |_, _| {});
    })
}

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder);
    let data = Default::default();
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
}
