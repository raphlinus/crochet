//! A widget with predefined size.

use crate::{view, DruidAppData, MutableWidget, MutationIter, Payload, SingleChild};
use druid::{widget::prelude::*, Point};

/// A widget with predefined size.
///
/// If given a child, this widget forces its child to have a specific width and/or height
/// (assuming values are permitted by this widget's parent). If either the width or height is not set,
/// this widget will size itself to match the child's size in that dimension.
///
/// If not given a child, SizedBox will try to size itself as close to the specified height
/// and width as possible given the parent's constraints. If height or width is not set,
/// it will be treated as zero.
pub struct SizedBox {
    width: Option<f64>,
    height: Option<f64>,
    inner: SingleChild,
}

impl MutableWidget for SizedBox {
    fn mutate(&mut self, ctx: &mut EventCtx, body: Option<&Payload>, mut_iter: MutationIter) {
        if let Some(Payload::View(view)) = body {
            if let Some(v) = view.as_any().downcast_ref::<view::SizedBox>() {
                self.width = v.width;
                self.height = v.height;
                ctx.request_layout();
            }
        }

        self.inner.mutate(ctx, mut_iter);
    }
}

impl SizedBox {
    pub fn new(view: &view::SizedBox) -> Self {
        Self {
            width: view.width,
            height: view.height,
            inner: SingleChild::new(),
        }
    }

    fn child_constraints(&self, bc: &BoxConstraints) -> BoxConstraints {
        // if we don't have a width/height, we don't change that axis.
        // if we have a width/height, we clamp it on that axis.
        let (min_width, max_width) = match self.width {
            Some(width) => {
                let w = width.max(bc.min().width).min(bc.max().width);
                (w, w)
            }
            None => (bc.min().width, bc.max().width),
        };

        let (min_height, max_height) = match self.height {
            Some(height) => {
                let h = height.max(bc.min().height).min(bc.max().height);
                (h, h)
            }
            None => (bc.min().height, bc.max().height),
        };

        BoxConstraints::new(
            Size::new(min_width, min_height),
            Size::new(max_width, max_height),
        )
    }
}

impl Widget<DruidAppData> for SizedBox {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut DruidAppData, env: &Env) {
        if let Some(inner) = self.inner.get_mut() {
            inner.event(ctx, event, data, env);
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &DruidAppData,
        env: &Env,
    ) {
        if let Some(inner) = self.inner.get_mut() {
            inner.lifecycle(ctx, event, data, env)
        }
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        _old_data: &DruidAppData,
        data: &DruidAppData,
        env: &Env,
    ) {
        if let Some(inner) = self.inner.get_mut() {
            inner.update(ctx, data, env);
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &DruidAppData,
        env: &Env,
    ) -> Size {
        bc.debug_check("SizedBox");

        let child_bc = self.child_constraints(bc);
        let size = match self.inner.get_mut() {
            Some(inner) => {
                let size = inner.layout(ctx, &child_bc, data, env);
                inner.set_origin(ctx, data, env, Point::ZERO);
                size
            }
            None => bc.constrain((self.width.unwrap_or(0.0), self.height.unwrap_or(0.0))),
        };

        if size.width.is_infinite() {
            log::warn!("SizedBox is returning an infinite width.");
        }

        if size.height.is_infinite() {
            log::warn!("SizedBox is returning an infinite height.");
        }

        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &DruidAppData, env: &Env) {
        if let Some(inner) = self.inner.get_mut() {
            inner.paint(ctx, data, env);
        }
    }

    fn id(&self) -> Option<WidgetId> {
        self.inner.get().map(|inner| inner.id())
    }
}
