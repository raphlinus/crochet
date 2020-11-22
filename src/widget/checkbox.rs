use crate::{any_widget::Action, view, DruidAppData, Id, MutableWidget, MutationIter, Payload};
use druid::{widget::prelude::*, WidgetPod};

/// A wrapper around `druid::Checkbox` with `DruidAppData` instead of `bool`.
pub struct Checkbox {
    id: Id,
    state: bool,
    inner: WidgetPod<bool, druid::widget::Checkbox>,
}

impl Checkbox {
    pub fn new(id: Id, state: bool, label: String) -> Self {
        let inner = WidgetPod::new(druid::widget::Checkbox::new(label));
        Checkbox { id, state, inner }
    }
}

impl MutableWidget for Checkbox {
    fn mutate(&mut self, ctx: &mut EventCtx, body: Option<&Payload>, _mut_iter: MutationIter) {
        if let Some(Payload::View(view)) = body {
            if let Some(v) = view.as_any().downcast_ref::<view::Checkbox>() {
                self.state = v.state;
                self.inner.widget_mut().set_text(v.label.clone());
                ctx.request_update();
            }
        }
    }
}

impl druid::Widget<DruidAppData> for Checkbox {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut DruidAppData, env: &Env) {
        let mut state = self.state;
        self.inner.event(ctx, event, &mut state, env);
        if state != self.state {
            data.queue_action(self.id, Action::Toggled(state));
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        _data: &DruidAppData,
        env: &Env,
    ) {
        self.inner.lifecycle(ctx, event, &self.state, env);
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        _old_data: &DruidAppData,
        _data: &DruidAppData,
        env: &Env,
    ) {
        self.inner.update(ctx, &self.state, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &DruidAppData,
        env: &Env,
    ) -> Size {
        let size = self.inner.layout(ctx, bc, &self.state, env);
        self.inner
            .set_layout_rect(ctx, &self.state, env, size.to_rect());
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &DruidAppData, env: &Env) {
        self.inner.paint(ctx, &self.state, env);
    }
}
