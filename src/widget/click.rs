use druid::{widget::prelude::*, MouseButton, Point};

use crate::{
    any_widget::Action, DruidAppData, Id, MutableWidget, MutationIter, Payload, SingleChild,
};

pub struct Click {
    id: Id,
    child: SingleChild,
}

impl Click {
    pub fn new(id: Id) -> Self {
        Click {
            id,
            child: SingleChild::new(),
        }
    }
}

impl MutableWidget for Click {
    fn mutate(&mut self, ctx: &mut EventCtx, _body: Option<&Payload>, mut_iter: MutationIter) {
        self.child.mutate(ctx, mut_iter);
    }
}

impl Widget<DruidAppData> for Click {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut DruidAppData, env: &Env) {
        match event {
            Event::MouseDown(mouse_event) => {
                if mouse_event.button == MouseButton::Left {
                    ctx.set_active(true);
                    ctx.request_paint();
                }
            }
            Event::MouseUp(mouse_event) => {
                if ctx.is_active() && mouse_event.button == MouseButton::Left {
                    ctx.set_active(false);
                    if ctx.is_hot() {
                        data.queue_action(self.id, Action::Clicked);
                    }
                    ctx.request_paint();
                }
            }
            _ => {}
        }

        if let Some(child) = self.child.get_mut() {
            child.event(ctx, event, data, env);
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &DruidAppData,
        env: &Env,
    ) {
        if let LifeCycle::HotChanged(_) | LifeCycle::FocusChanged(_) = event {
            ctx.request_paint();
        }

        if let Some(child) = self.child.get_mut() {
            child.lifecycle(ctx, event, data, env);
        }
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        _old_data: &DruidAppData,
        data: &DruidAppData,
        env: &Env,
    ) {
        if let Some(child) = self.child.get_mut() {
            child.update(ctx, data, env);
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &DruidAppData,
        env: &Env,
    ) -> Size {
        if let Some(child) = self.child.get_mut() {
            let size = child.layout(ctx, bc, data, env);
            child.set_origin(ctx, data, env, Point::ZERO);
            size
        } else {
            Size::ZERO
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &DruidAppData, env: &Env) {
        if let Some(child) = self.child.get_mut() {
            child.paint(ctx, data, env);
        }
    }
}
