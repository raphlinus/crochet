//! A Druid widget that contains the application.

use std::collections::HashMap;

use druid::widget::prelude::*;
use druid::{Point, Selector, SingleUse, WidgetPod};

use crate::any_widget::{Action, AnyWidget, DruidAppData};
use crate::state::State;
use crate::{Cx, Id, MutationIter, Tree};

pub const ASYNC: Selector<SingleUse<(Id, Id, Box<dyn State>)>> = Selector::new("crochet.async");

/// A container for a user application.
///
/// In the prototype, this container is a Druid widget, to be
/// placed at the top of the widget tree. It contains the Crochet
/// tree and the closure for running the user application code.
///
/// As a Druid widget, it takes no app data; in the Crochet
/// architecture, that is stored in the app logic closure and the
/// Crochet tree instead.
///
/// Right now I'm only thinking about the single window case. For
/// multi-window, this should probably be an app delegate.
pub struct AppHolder {
    tree: Tree,
    /// The app logic.
    ///
    /// It's a choice whether to box this or not. The argument in
    /// favor is simpler types and less monomorphization.
    app_logic: Box<dyn FnMut(&mut Cx)>,
    child: WidgetPod<DruidAppData, AnyWidget>,

    /// This is where the values of resolved futures are stored. It's
    /// not ideal, as they are not garbage collected when the Id is
    /// removed from the tree. A better idea is probably to store them
    /// in the tree, but that involves more ceremony, especially around
    /// ownership.
    resolved_futures: HashMap<Id, Box<dyn State>>,
}

impl AppHolder {
    pub fn new(app_logic: impl FnMut(&mut Cx) + 'static) -> AppHolder {
        let child = WidgetPod::new(AnyWidget::column());
        AppHolder {
            tree: Tree::default(),
            app_logic: Box::new(app_logic),
            child,
            resolved_futures: Default::default(),
        }
    }

    /// Run the app logic, mutating the tree.
    ///
    /// We cheat slightly, only implementing this for event contexts.
    /// This is probably good enough for a prototype, but will probably
    /// need more care for a real integration.
    fn run_app_logic(&mut self, ctx: &mut EventCtx, data: &mut DruidAppData) {
        let needs_update = data.has_any_action();
        let event_sink = ctx.get_external_handle();
        let mut cx = Cx::new(&self.tree, data, &self.resolved_futures, &event_sink);
        (self.app_logic)(&mut cx);
        let mutation = cx.into_mutation();
        let mut_iter = MutationIter::new(&self.tree, &mutation);
        self.child.widget_mut().mutate_update(ctx, None, mut_iter);
        self.tree.mutate(mutation);
        // This will bring the ui up-to-date and avoid stale state.
        // A better solution would be nice, but this is simple and seems to work.
        if needs_update {
            ctx.request_timer(std::time::Duration::from_secs(0));
        }
    }
}

impl Widget<DruidAppData> for AppHolder {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut DruidAppData, env: &Env) {
        if let Event::Command(cmd) = event {
            if let Some(payload) = cmd.get(ASYNC) {
                if let Some((id, f_id, val)) = payload.take() {
                    self.resolved_futures.insert(f_id, val);
                    data.queue_action(id, Action::FutureResolved);
                }
            }
        }
        self.child.event(ctx, event, data, env);
        self.run_app_logic(ctx, data);
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &DruidAppData,
        env: &Env,
    ) {
        //println!("lifecycle: {:?}", event);
        self.child.lifecycle(ctx, event, data, env);
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        _old_data: &DruidAppData,
        data: &DruidAppData,
        env: &Env,
    ) {
        //println!("update");
        self.child.update(ctx, data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &DruidAppData,
        env: &Env,
    ) -> Size {
        //println!("layout, bc={:?}", bc);
        let size = self.child.layout(ctx, bc, data, env);
        self.child
            .set_layout_rect(ctx, data, env, (Point::ZERO, size).into());
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &DruidAppData, env: &Env) {
        //println!("paint");
        self.child.paint(ctx, data, env);
    }
}
