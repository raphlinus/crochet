use druid::widget::prelude::*;
use druid::widget::Button;

/// A widget that backs any render element in the crochet tree.
///
/// This is something of a hack to add a method to the Druid `Widget`
/// trait, and exists for convenience of prototyping.
///
/// In the expected evolution of the architecture, the `mutate`
/// method is added to `Widget`.
pub enum AnyWidget {
    Button(Button<()>),
}

impl AnyWidget {
    /// Create a new button.
    ///
    /// Note: at present, the label doesn't update. This can be fixed.
    pub fn button(label: &str) -> AnyWidget {
        let button = Button::new(label);
        AnyWidget::Button(button)
    }
}

macro_rules! methods {
    ($method_name: ident, $self: ident, $($args:ident),+) => {
        match $self {
            AnyWidget::Button(b) => b.$method_name($($args),+),
        }
    };
}

impl Widget<()> for AnyWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut (), env: &Env) {
        methods!(event, self, ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &(), env: &Env) {
        methods!(lifecycle, self, ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &(), data: &(), env: &Env) {
        methods!(update, self, ctx, old_data, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &(), env: &Env) -> Size {
        methods!(layout, self, ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &(), env: &Env) {
        methods!(paint, self, ctx, data, env);
    }
}
