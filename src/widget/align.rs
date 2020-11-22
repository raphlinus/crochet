//! A widget that aligns its child (for example, centering it).

use crate::{view, DruidAppData, MutableWidget, Payload, SingleChild};
use druid::widget::prelude::*;
use druid::{Rect, Size};

/// A widget that aligns its child.
pub struct Align {
    view: view::Align,
    child: SingleChild,
}

impl Align {
    pub fn new(view: &view::Align) -> Self {
        Align {
            view: view.clone(),
            child: SingleChild::new(),
        }
    }
}

impl MutableWidget for Align {
    fn mutate(
        &mut self,
        ctx: &mut EventCtx,
        body: Option<&crate::Payload>,
        mut_iter: crate::MutationIter,
    ) {
        if let Some(Payload::View(view)) = body {
            if let Some(v) = view.as_any().downcast_ref::<view::Align>() {
                self.view = v.clone();
                ctx.request_layout();
            }
        }

        self.child.mutate(ctx, mut_iter);
    }
}

impl Widget<DruidAppData> for Align {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut DruidAppData, env: &Env) {
        if let Some(child) = self.child.get_mut() {
            child.event(ctx, event, data, env)
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &DruidAppData,
        env: &Env,
    ) {
        if let Some(child) = self.child.get_mut() {
            child.lifecycle(ctx, event, data, env)
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
        bc.debug_check("Align");

        let size = self
            .child
            .get_mut()
            .map(|child| child.layout(ctx, &bc.loosen(), data, env))
            .unwrap_or_default();

        log_size_warnings(size);

        let mut my_size = size;
        if bc.is_width_bounded() {
            my_size.width = bc.max().width;
        }
        if bc.is_height_bounded() {
            my_size.height = bc.max().height;
        }

        if let Some(width) = self.view.width_factor {
            my_size.width = size.width * width;
        }
        if let Some(height) = self.view.height_factor {
            my_size.height = size.height * height;
        }

        my_size = bc.constrain(my_size);
        let extra_width = (my_size.width - size.width).max(0.);
        let extra_height = (my_size.height - size.height).max(0.);
        let origin = self
            .view
            .align
            .resolve(Rect::new(0., 0., extra_width, extra_height))
            .expand();
        if let Some(child) = self.child.get_mut() {
            child.set_origin(ctx, data, env, origin);
        }

        let my_insets = self
            .child
            .get_mut()
            .map(|child| child.compute_parent_paint_insets(my_size))
            .unwrap_or_default();
        ctx.set_paint_insets(my_insets);
        my_size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &DruidAppData, env: &Env) {
        if let Some(child) = self.child.get_mut() {
            child.paint(ctx, data, env);
        }
    }
}

fn log_size_warnings(size: Size) {
    if size.width.is_infinite() {
        log::warn!("Align widget's child has an infinite width.");
    }

    if size.height.is_infinite() {
        log::warn!("Align widget's child has an infinite height.");
    }
}
