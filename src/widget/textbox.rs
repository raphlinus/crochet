use crate::{any_widget::Action, view, DruidAppData, Id, MutableWidget, MutationIter, Payload};
use druid::{widget::prelude::*, WidgetPod};

/// A wrapper around `druid::TextBox` with `DruidAppData` instead of `String`.
pub struct TextBox {
    id: Id,
    content: String,
    inner: WidgetPod<String, druid::widget::TextBox<String>>,
}

impl TextBox {
    pub fn new(id: Id, content: String, inner: druid::widget::TextBox<String>) -> Self {
        let inner = WidgetPod::new(inner);
        TextBox { id, content, inner }
    }
}

impl MutableWidget for TextBox {
    fn mutate(&mut self, ctx: &mut EventCtx, body: Option<&Payload>, _mut_iter: MutationIter) {
        if let Some(Payload::View(view)) = body {
            if let Some(v) = view.as_any().downcast_ref::<view::TextBox>() {
                self.content = v.0.clone();
                ctx.request_update();
            }
        }
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
