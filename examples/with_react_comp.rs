#[allow(unused_imports)]
use crochet::react_comp::{ReactComponent, ComponentTuple, ComponentList, VDomLeaf, VirtualDom, EmptyComponent};
use crochet::react_comp::{ButtonPressed, EventEnum};
use crochet::react_builder::{ComponentBuilder, ComponentTupleBuilder, ComponentListBuilder, VDomLeafBuilder, VirtualDomBuilder, WithState};

#[allow(unused_imports)]
use crochet::{AppHolder, Button, Cx, DruidAppData, Id, Label, List, ListData, Row};

use druid::{AppLauncher, PlatformError, Widget, WindowDesc};


#[derive(Debug, Clone)]
struct ListItem {
    text: String,
    id: i32,
}

struct Props {
    data: Vec<ListItem>,
    selected_row: Option<i32>,
    next_id: i32,
}


struct RowProps<'a> {
    list_item: &'a ListItem,
    is_selected: bool,
}
fn list_row(_state: &(), props: RowProps) ->
    ComponentTuple<VDomLeaf, VDomLeaf, VDomLeaf, VDomLeaf>
{
    ComponentTuple(
        VDomLeaf::Button("Select".into()),
        VDomLeaf::Label(if props.is_selected { "[*]".into() } else { "[ ]".into() }),
        VDomLeaf::Label(props.list_item.text.clone()),
        VDomLeaf::Label(props.list_item.id.to_string()),
    )
}

type EventType = EventEnum<
    VDomLeaf,
    VDomLeaf,
    VDomLeaf,
    ComponentList<WithState<ComponentTuple<VDomLeaf, VDomLeaf, VDomLeaf, VDomLeaf>, ()>>,
>;

fn some_component(_state: &(), props: &Props) -> impl VirtualDom<Event = EventType> {
    let button_create = VDomLeaf::Button("Create".into());
    let button_delete = VDomLeaf::Button("Delete".into());
    let button_update = VDomLeaf::Button("Update".into());

    let list_view_data = props.data.iter().enumerate().map(|(i, list_item)| {
        let row_props = RowProps {
            list_item: &list_item,
            is_selected: i as i32 == props.selected_row.unwrap_or(-1),
        };
        let comp_builder = ComponentBuilder {
            component: list_row,
            props: row_props,
            _vdom: Default::default(),
            _state: Default::default(),
        };

        (list_item.id.to_string(), comp_builder)
    }).collect();
    let list_view = ComponentListBuilder { components: list_view_data };

    ComponentTupleBuilder(
        VDomLeafBuilder(button_create),
        VDomLeafBuilder(button_delete),
        VDomLeafBuilder(button_update),
        list_view,
    ).build()
}



fn ui_builder() -> impl Widget<DruidAppData> {
    let mut react_component = ReactComponent {
        root_component: &some_component,
        prev_vdom: None,
        prev_vdom_state: None,
        _props: Default::default(),
    };
    let mut props = Props {
        data: (0..8_i32).map(|i| ListItem { text: "hello".to_string(), id: i }).collect(),
        selected_row: None,
        next_id: 8,
    };

    AppHolder::new(move |cx| {
        react_component.run(cx, &mut props, |event, props| {
            match event {
                EventEnum::E0(ButtonPressed()) => {
                    props.data.push(ListItem { text: "new item".to_string(), id: props.next_id });
                    props.next_id += 1;
                },
                EventEnum::E1(ButtonPressed()) => {
                    if let Some(row) = props.selected_row {
                        props.data.remove(row as usize);
                        props.selected_row = None;
                    }
                },
                EventEnum::E2(ButtonPressed()) => {
                    if let Some(row) = props.selected_row {
                        props.data[row as usize].text = "updated".to_string();
                    }
                },
                EventEnum::E3((i, EventEnum::E0(ButtonPressed()))) => {
                    props.selected_row = Some(*i);
                },
                _ => {},
            }
        });
    })
}

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder);
    let data = Default::default();
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
}
