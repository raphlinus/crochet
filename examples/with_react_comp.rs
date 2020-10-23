#[allow(unused_imports)]
use crochet::react_comp::{ReactComponent, ComponentTuple, ComponentList, VDomLabel, VDomButton, VirtualDom, EmptyComponent};
use crochet::react_comp::{ButtonPressed, EventEnum};
use crochet::react_builder::{ComponentBuilder, ComponentTupleBuilder, ComponentListBuilder, VDomLabelBuilder, VDomButtonBuilder, VirtualDomBuilder, WithEventBuilder};

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
    ComponentTupleBuilder(
        VDomButtonBuilder(VDomButton("Select".into(), Default::default())),
        VDomLabelBuilder(VDomLabel(if props.is_selected { "[*]".into() } else { "[ ]".into() }, Default::default())),
        VDomLabelBuilder(VDomLabel(props.list_item.text.clone(), Default::default())),
        VDomLabelBuilder(VDomLabel(props.list_item.id.to_string(), Default::default())),
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
    let button_create = VDomButton("Create".into(), Default::default());
    let button_delete = VDomButton("Delete".into(), Default::default());
    let button_update = VDomButton("Update".into(), Default::default());

    let list_view_data = state.data.iter().enumerate().map(|(i, list_item)| {
        let row_props = RowProps {
            list_item: &list_item,
            is_selected: i as i32 == state.selected_row.unwrap_or(-1),
        };

        let comp_builder = ComponentBuilder {
            component: list_row,
            props: row_props,
            _vdom: Default::default(),
            _state: Default::default(),
            _expl_state: Default::default(),
        };

        (list_item.id.to_string(), comp_builder)
    }).collect();
    let list_view = WithEventBuilder {
        component: ComponentListBuilder {
            components: list_view_data, _state: Default::default()
        },
        callback: |state: &mut AppState, event| {
            state.selected_row = Some(event.0);
        },
        _state: Default::default(),
    };

    ComponentTupleBuilder(
        WithEventBuilder {
            component: VDomButtonBuilder(button_create),
            callback: |state: &mut AppState, _| {
                state.data.push(ListItem { text: "new item".to_string(), id: state.next_id });
                state.next_id += 1;
            },
            _state: Default::default(),
        },
        WithEventBuilder {
            component: VDomButtonBuilder(button_delete),
            callback: |state: &mut AppState, _| {
                if let Some(row) = state.selected_row {
                    state.data.remove(row as usize);
                    state.selected_row = None;
                }
            },
            _state: Default::default(),
        },
        WithEventBuilder {
            component: VDomButtonBuilder(button_update),
            callback: |state: &mut AppState, _| {
                if let Some(row) = state.selected_row {
                    state.data[row as usize].text = "updated".to_string();
                }
            },
            _state: Default::default(),
        },
        list_view,
        Default::default(),
    ).build()
}


fn ui_builder() -> impl Widget<DruidAppData> {
    let state = AppState {
        data: (0..8_i32).map(|i| ListItem { text: "hello".to_string(), id: i }).collect(),
        selected_row: None,
        next_id: 8,
    };

    let mut react_component = ReactComponent::<AppState, (), _, _> {
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
