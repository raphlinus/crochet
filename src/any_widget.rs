use druid::widget::prelude::*;
use druid::widget::{Button, Click, ControllerHost};

use crate::flex::Flex;
use crate::{Id, MutationIter};

/// The type we use for app data for Druid integration.
///
/// It is currently empty, but it's possible we'll want to put
/// action queues here.
pub type DruidAppData = ();

/// A widget that backs any render element in the crochet tree.
///
/// This is something of a hack to add a method to the Druid `Widget`
/// trait, and exists for convenience of prototyping.
///
/// In the expected evolution of the architecture, the `mutate`
/// method is added to `Widget`.
pub enum AnyWidget {
    Button(ControllerHost<Button<DruidAppData>, Click<DruidAppData>>),
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
        _body: Option<&String>,
        mut_iter: MutationIter,
    ) {
        match self {
            AnyWidget::Button(_) => (),
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
                let button = Button::new(args.unwrap_or("Button"))
                    .on_click(move |_, _, _| println!("button {:?} clicked", id));
                AnyWidget::Button(button)
            }
            "row" => AnyWidget::Flex(Flex::row()),
            "column" => AnyWidget::Flex(Flex::column()),
            _ => panic!("unknown widget type {}", widget_type),
        }
    }
}
