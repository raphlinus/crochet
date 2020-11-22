use crate::{view, DruidAppData, MutableWidget, MutationIter, Payload};
use druid::{widget::prelude::*, Data};

pub struct Painter<D> {
    view: view::Painter<D>,
}

impl<D: Data> Painter<D> {
    pub fn new(view: view::Painter<D>) -> Self {
        Painter { view }
    }
}

impl<T: Data> MutableWidget for Painter<T> {
    fn mutate(&mut self, ctx: &mut EventCtx, body: Option<&Payload>, _: MutationIter) {
        if let Some(Payload::View(view)) = body {
            if let Some(v) = view.as_any().downcast_ref::<view::Painter<T>>() {
                self.view = v.clone();
                ctx.request_paint();
            }
        }
    }
}

impl<T: Data> Widget<DruidAppData> for Painter<T> {
    fn event(&mut self, _: &mut EventCtx, _: &Event, _: &mut DruidAppData, _: &Env) {}
    fn lifecycle(&mut self, _: &mut LifeCycleCtx, _: &LifeCycle, _: &DruidAppData, _: &Env) {}
    fn update(&mut self, _: &mut UpdateCtx, _: &DruidAppData, _: &DruidAppData, _: &Env) {}
    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _: &DruidAppData,
        _: &Env,
    ) -> Size {
        bc.max()
    }
    fn paint(&mut self, ctx: &mut PaintCtx, _: &DruidAppData, env: &Env) {
        let data = &self.view.data;
        (self.view.paint)(ctx, env, data)
    }
}
