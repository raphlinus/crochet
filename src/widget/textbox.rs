use crate::{any_widget::Action, DruidAppData, Id};
use druid::{widget::prelude::*, WidgetPod};

/// A wrapper around `druid::TextBox` with `DruidAppData` instead of `String`.
pub struct TextBox {
    id: Id,
    content: String,
    inner: WidgetPod<String, druid::widget::TextBox>,
}

impl TextBox {
    pub fn new(id: Id, content: String, inner: druid::widget::TextBox) -> Self {
        let inner = WidgetPod::new(inner);
        TextBox { id, content, inner }
    }

    pub fn set_text(&mut self, content: String) {
        self.content = content;
    }
}

impl druid::Widget<DruidAppData> for TextBox {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut DruidAppData, env: &Env) {
        let old_content = self.content.clone();
        self.inner.event(ctx, event, &mut self.content, env);
        if old_content != self.content {
            data.queue_action(self.id, Action::TextChanged(self.content.clone()));
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        _data: &DruidAppData,
        env: &Env,
    ) {
        self.inner.lifecycle(ctx, event, &self.content, env);
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        _old_data: &DruidAppData,
        _data: &DruidAppData,
        env: &Env,
    ) {
        self.inner.update(ctx, &self.content, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &DruidAppData,
        env: &Env,
    ) -> Size {
        let size = self.inner.layout(ctx, bc, &self.content, env);
        self.inner
            .set_layout_rect(ctx, &self.content, env, size.to_rect());
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &DruidAppData, env: &Env) {
        self.inner.paint(ctx, &self.content, env);
    }
}
