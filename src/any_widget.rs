use std::collections::HashMap;
use std::sync::Arc;

use druid::widget::prelude::*;
use druid::widget::{Button, Click, ControllerHost, Label};
use druid::Data;

use crate::widget::{Checkbox, Click as CrochetClick, Flex, Padding, TextBox};
use crate::{view, widget::SizedBox};
use crate::{Id, MutIterItem, MutationIter, Payload};

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
    FutureResolved,
    TextChanged(String),
    Toggled(bool),
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
    TextBox(TextBox),
    Padding(Padding),
    Checkbox(Checkbox),
    Click(CrochetClick),
    SizedBox(SizedBox),
    /// A do-nothing container for another widget.
    ///
    /// Currently we use this for state nodes.
    Passthrough(Box<AnyWidget>),
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
            AnyWidget::TextBox(w) => w.$method_name($($args),+),
            AnyWidget::Padding(w) => w.$method_name($($args),+),
            AnyWidget::Checkbox(w) => w.$method_name($($args),+),
            AnyWidget::Click(w) => w.$method_name($($args),+),
            AnyWidget::SizedBox(w) => w.$method_name($($args),+),
            AnyWidget::Passthrough(w) => w.$method_name($($args),+),
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
        body: Option<&Payload>,
        mut mut_iter: MutationIter,
    ) {
        match self {
            AnyWidget::Button(_) => (),
            AnyWidget::Label(l) => {
                if let Some(Payload::View(view)) = body {
                    if let Some(v) = view.as_any().downcast_ref::<view::Label>() {
                        l.set_text(v.0.to_string());
                        ctx.request_update();
                    }
                }
            }
            AnyWidget::Flex(f) => f.mutate(ctx, mut_iter),
            AnyWidget::TextBox(t) => {
                if let Some(Payload::View(view)) = body {
                    if let Some(v) = view.as_any().downcast_ref::<view::TextBox>() {
                        t.set_text(v.0.to_string());
                        ctx.request_update();
                    }
                }
            }
            AnyWidget::Padding(p) => p.mutate(ctx, body, mut_iter),
            AnyWidget::Checkbox(c) => {
                if let Some(Payload::View(view)) = body {
                    if let Some(v) = view.as_any().downcast_ref::<view::Checkbox>() {
                        c.set_text(v.label.to_string());
                        c.set_state(v.state);
                        ctx.request_update();
                    }
                }
            }
            AnyWidget::Click(c) => c.mutate(ctx, body, mut_iter),
            AnyWidget::SizedBox(b) => b.mutate(ctx, body, mut_iter),
            AnyWidget::Passthrough(p) => {
                if let Some(MutIterItem::Update(body, iter)) = mut_iter.next() {
                    p.mutate_update(ctx, body, iter);
                }
            }
        }
    }

    /// Create a new widget tree in response to a Crochet tree mutation insert request.
    pub(crate) fn mutate_insert(
        ctx: &mut EventCtx,
        id: Id,
        body: &Payload,
        mut mut_iter: MutationIter,
    ) -> AnyWidget {
        match body {
            Payload::View(v) => {
                let mut widget = v.make_widget(id);
                widget.mutate_update(ctx, None, mut_iter);
                widget
            }
            Payload::State(_) | Payload::Future(..) | Payload::Placeholder => {
                // Here we assume that the state node has exactly one
                // child. Not awesome but it simplifies prototyping.
                if let Some(MutIterItem::Insert(id, body, iter)) = mut_iter.next() {
                    let child = Self::mutate_insert(ctx, id, body, iter);
                    AnyWidget::Passthrough(Box::new(child))
                } else {
                    panic!("passthrough node expected child");
                }
            }
        }
    }
}

impl DruidAppData {
    pub(crate) fn queue_action(&mut self, id: Id, action: Action) {
        Arc::make_mut(&mut self.0).insert(id, action);
    }

    pub(crate) fn dequeue_action(&mut self, id: Id) -> Option<Action> {
        if self.0.contains_key(&id) {
            Arc::make_mut(&mut self.0).remove(&id)
        } else {
            None
        }
    }

    /// Report whether the id has a non-empty action queue.
    pub(crate) fn has_action(&self, id: Id) -> bool {
        self.0.contains_key(&id)
    }

    pub(crate) fn has_any_action(&self) -> bool {
        !self.0.is_empty()
    }
}
