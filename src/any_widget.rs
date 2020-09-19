use std::collections::HashMap;
use std::sync::Arc;

use druid::widget::prelude::*;
use druid::widget::{Button, Click, ControllerHost, Label};
use druid::Data;

use crate::flex::Flex;
use crate::{Id, MutationIter};

/// The type we use for app data for Druid integration.
///
/// Currently this is action queues.
///
/// It should probably be a vec of actions, but we can refine
/// later. For button clicks it doesn't matter.
#[derive(Clone, Data, Default)]
pub struct DruidAppData(Arc<HashMap<Id, Action>>);

/// Actions that can be produced by widgets,
#[derive(Clone)]
pub enum Action {
    Clicked,
}

/// A widget that backs any render element in the crochet tree.
///
/// This is something of a hack to add a method to the Druid `Widget`
/// trait, and exists for convenience of prototyping.
///
/// In the expected evolution of the architecture, the `mutate`
/// method is added to `Widget`.
pub enum AnyWidget {
    Button(ControllerHost<Button<DruidAppData>, Click<DruidAppData>>),
    Label(Label<DruidAppData>),
    Flex(Flex),
}

impl AnyWidget {
    /// Create a new column.
    pub fn column() -> AnyWidget {
        AnyWidget::Flex(Flex::column())
    }
}

macro_rules! methods {
    ($method_name: ident, $self: ident, $($args:ident),+) => {
        match $self {
            AnyWidget::Button(w) => w.$method_name($($args),+),
            AnyWidget::Label(w) => w.$method_name($($args),+),
            AnyWidget::Flex(w) => w.$method_name($($args),+),
        }
    };
}

impl Widget<DruidAppData> for AnyWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut DruidAppData, env: &Env) {
        methods!(event, self, ctx, event, data, env);
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &DruidAppData,
        env: &Env,
    ) {
        methods!(lifecycle, self, ctx, event, data, env);
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &DruidAppData,
        data: &DruidAppData,
        env: &Env,
    ) {
        methods!(update, self, ctx, old_data, data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &DruidAppData,
        env: &Env,
    ) -> Size {
        methods!(layout, self, ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &DruidAppData, env: &Env) {
        methods!(paint, self, ctx, data, env);
    }
}

impl AnyWidget {
    /// Mutate the widget tree in response to a Crochet tree mutation update request.
    pub(crate) fn mutate_update(
        &mut self,
        ctx: &mut EventCtx,
        body: Option<&String>,
        mut_iter: MutationIter,
    ) {
        match self {
            AnyWidget::Button(_) => (),
            AnyWidget::Label(l) => {
                if let Some(descr) = body {
                    if let Some(text) = descr.splitn(2, ": ").skip(1).next() {
                        l.set_text(text.to_string());
                    }
                    ctx.request_layout();
                }
            }
            AnyWidget::Flex(f) => f.mutate(ctx, mut_iter),
        }
    }

    /// Create a new widget tree in reponse to a Crochet tree mutation insert request.
    pub(crate) fn mutate_insert(
        ctx: &mut EventCtx,
        id: Id,
        body: &str,
        mut_iter: MutationIter,
    ) -> AnyWidget {
        let mut widget = AnyWidget::create(id, body);
        widget.mutate_update(ctx, None, mut_iter);
        widget
    }

    /// Create a new widget.
    ///
    /// This is stringly-typed for expedience; that will change.
    fn create(id: Id, descr: &str) -> AnyWidget {
        let mut split_iter = descr.splitn(2, ": ");
        let widget_type = split_iter.next().unwrap();
        let args = split_iter.next();
        match widget_type {
            "button" => {
                let button = Button::new(args.unwrap_or("Button")).on_click(
                    move |_, data: &mut DruidAppData, _| data.queue_action(id, Action::Clicked),
                );
                AnyWidget::Button(button)
            }
            "label" => {
                let label = Label::new(args.unwrap_or("Label"));
                AnyWidget::Label(label)
            }
            "row" => AnyWidget::Flex(Flex::row()),
            "column" => AnyWidget::Flex(Flex::column()),
            _ => panic!("unknown widget type {}", widget_type),
        }
    }
}

impl DruidAppData {
    fn queue_action(&mut self, id: Id, action: Action) {
        Arc::make_mut(&mut self.0).insert(id, action);
    }

    pub(crate) fn dequeue_action(&mut self, id: Id) -> Option<Action> {
        if self.0.contains_key(&id) {
            Arc::make_mut(&mut self.0).remove(&id)
        } else {
            None
        }
    }
}
