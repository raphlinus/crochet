use crate::{any_widget::Action, DruidAppData, Id};
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

    pub fn set_state(&mut self, state: bool) {
        self.state = state;
    }

    pub fn set_text(&mut self, label: String) {
        self.inner.widget_mut().set_text(label);
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
