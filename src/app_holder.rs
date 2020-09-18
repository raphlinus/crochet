//! A Druid widget that contains the application.

use druid::{Point, WidgetPod};
use druid::widget::prelude::*;
use druid::widget::Button;

use crate::{MutCursor, Tree};
use crate::any_widget::AnyWidget;

/// A container for a user application.
///
/// In the prototype, this container is a Druid widget, to be
/// placed at the top of the widget tree. It contains the Crochet
/// tree and the closure for running the user application code.
///
/// As a Druid widget, it takes no app data; in the Crochet
/// architecture, that is stored in the app logic closure and the
/// Crochet tree instead.
pub struct AppHolder {
    tree: Tree,
    /// The app logic.
    ///
    /// It's a choice whether to box this or not. The argument in
    /// favor is simpler types and less monomorphization.
    app_logic: Box<dyn FnMut(&MutCursor)>,
    child: WidgetPod<(), AnyWidget>,
}

impl AppHolder {
    pub fn new(app_logic: impl FnMut(&MutCursor) + 'static) -> AppHolder {
        let button = AnyWidget::button("Foo");
        let child = WidgetPod::new(button);
        AppHolder {
            tree: Tree::default(),
            app_logic: Box::new(app_logic),
            child,
        }
    }
}

impl Widget<()> for AppHolder {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut (), env: &Env) {
        println!("event: {:?}", event);
        self.child.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &(), env: &Env) {
        println!("lifecycle: {:?}", event);
        self.child.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &(), data: &(), env: &Env) {
        println!("update");
        self.child.update(ctx, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &(), env: &Env) -> Size {
        println!("layout, bc={:?}", bc);
        let size = self.child.layout(ctx, bc, data, env);
        self.child.set_layout_rect(ctx, data, env, (Point::ZERO, size).into());
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &(), env: &Env) {
        println!("paint");
        self.child.paint(ctx, data, env);
    }
}
