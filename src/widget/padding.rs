// Copyright 2018 The Druid Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A widget that just adds padding during layout.

use druid::kurbo::{Insets, Point, Rect, Size};
use druid::{
    BoxConstraints, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, UpdateCtx,
    Widget, WidgetPod,
};

use crate::{any_widget::AnyWidget, view, DruidAppData, MutIterItem, Payload};

/// A widget that just adds padding around its child.
pub struct Padding {
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,

    children: Vec<WidgetPod<DruidAppData, AnyWidget>>,
}

impl Padding {
    pub(crate) fn mutate(
        &mut self,
        ctx: &mut EventCtx,
        body: Option<&Payload>,
        mut_iter: crate::MutationIter,
    ) {
        if let Some(Payload::View(view)) = body {
            if let Some(v) = view.as_any().downcast_ref::<view::Padding>() {
                let insets = v.insets;
                self.left = insets.x0;
                self.right = insets.x1;
                self.top = insets.y0;
                self.bottom = insets.y1;

                ctx.request_layout();
            }
        }

        let mut ix = 0;
        let mut children_changed = false;
        for item in mut_iter {
            match item {
                MutIterItem::Skip(n) => {
                    println!("skipping {} items", n);
                    ix += n;
                }
                MutIterItem::Delete(n) => {
                    println!("deleting {} items", n);
                    self.children.drain(ix..ix + n);
                    children_changed = true;
                }
                MutIterItem::Insert(id, body, child_iter) => {
                    let child = AnyWidget::mutate_insert(ctx, id, body, child_iter);
                    self.children.insert(ix, WidgetPod::new(child));
                    ix += 1;
                    children_changed = true;
                }
                MutIterItem::Update(body, child_iter) => {
                    self.children[ix].with_event_context(ctx, |child, ctx| {
                        child.mutate_update(ctx, body, child_iter);
                    });
                    ix += 1;
                }
            }
        }
        if children_changed {
            ctx.children_changed();
        }
    }
}

impl Padding {
    /// Create a new widget with the specified padding. This can either be an instance
    /// of [`kurbo::Insets`], a f64 for uniform padding, a 2-tuple for axis-uniform padding
    /// or 4-tuple with (left, top, right, bottom) values.
    ///
    /// # Examples
    ///
    /// Uniform padding:
    ///
    /// ```
    /// use druid::widget::{Label, Padding};
    /// use druid::kurbo::Insets;
    ///
    /// let _: Padding<()> = Padding::new(10.0, Label::new("uniform!"));
    /// let _: Padding<()> = Padding::new(Insets::uniform(10.0), Label::new("uniform!"));
    /// ```
    ///
    /// Uniform padding across each axis:
    ///
    /// ```
    /// use druid::widget::{Label, Padding};
    /// use druid::kurbo::Insets;
    ///
    /// let child: Label<()> = Label::new("I need my space!");
    /// let _: Padding<()> = Padding::new((10.0, 20.0), Label::new("more y than x!"));
    /// // equivalent:
    /// let _: Padding<()> = Padding::new(Insets::uniform_xy(10.0, 20.0), Label::new("ditto :)"));
    /// ```
    ///
    /// [`kurbo::Insets`]: https://docs.rs/kurbo/0.5.3/kurbo/struct.Insets.html
    pub fn new(insets: impl Into<Insets>) -> Self {
        let insets = insets.into();
        Padding {
            left: insets.x0,
            right: insets.x1,
            top: insets.y0,
            bottom: insets.y1,
            children: Vec::new(),
        }
    }
}

impl Widget<DruidAppData> for Padding {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut DruidAppData, env: &Env) {
        if let Some(child) = self.children.get_mut(0) {
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
        if let Some(child) = self.children.get_mut(0) {
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
        if let Some(child) = self.children.get_mut(0) {
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
        bc.debug_check("Padding");
        if self.children.len() > 1 {
            log::warn!(
                "Padding only supports a single child, but it got {}",
                self.children.len()
            );
        }

        if let Some(child) = self.children.get_mut(0) {
            let hpad = self.left + self.right;
            let vpad = self.top + self.bottom;

            let child_bc = bc.shrink((hpad, vpad));
            let size = child.layout(ctx, &child_bc, data, env);
            let origin = Point::new(self.left, self.top);
            child.set_layout_rect(ctx, data, env, Rect::from_origin_size(origin, size));

            let my_size = Size::new(size.width + hpad, size.height + vpad);
            let my_insets = child.compute_parent_paint_insets(my_size);
            ctx.set_paint_insets(my_insets);
            my_size
        } else {
            Size::ZERO
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &DruidAppData, env: &Env) {
        if let Some(child) = self.children.get_mut(0) {
            child.paint(ctx, data, env);
        }
    }
}
