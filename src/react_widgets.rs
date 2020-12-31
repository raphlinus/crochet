use druid::kurbo::{Rect, Size};

use druid::{
    BoxConstraints, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx,
    PaintCtx, UpdateCtx, Widget, WidgetPod,
};
use crate::flex2::FlexParams;


pub trait FlexWidget {
    fn widget(&mut self) -> &mut dyn Widget<()>;
    fn flex_params(&self) -> FlexParams;

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, env: &Env) -> Size;
    fn paint_rect(&self) -> Rect;
    fn set_layout_rect(&mut self, ctx: &mut LayoutCtx, env: &Env, rect: Rect);
    fn layout_rect(&self) -> Rect;
    fn paint(&mut self, ctx: &mut PaintCtx, env: &Env);
}

pub trait WidgetSequence {
    fn widgets(&mut self) -> Vec<&mut dyn FlexWidget>;

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, env: &Env);
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, env: &Env);
    fn update(&mut self, ctx: &mut UpdateCtx, env: &Env);
}


pub struct SingleWidget<W: Widget<()>>(pub WidgetPod<(), W>);

impl<W: Widget<()>> FlexWidget for SingleWidget<W> {
    fn widget(&mut self) -> &mut dyn Widget<()> {
        self.0.widget_mut()
    }

    fn flex_params(&self) -> FlexParams {
        FlexParams {
            flex: 1.0,
            alignment: None,
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, env: &Env) -> Size {
        self.0.layout(ctx, bc, &(), env)
    }

    fn paint_rect(&self) -> Rect {
        self.0.paint_rect()
    }

    fn set_layout_rect(&mut self, ctx: &mut LayoutCtx, env: &Env, rect: Rect) {
        self.0.set_layout_rect(ctx, &(), env, rect)
    }

    fn layout_rect(&self) -> Rect {
        self.0.layout_rect()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, env: &Env) {
        self.0.paint(ctx, &(), env);
    }
}

impl<W: Widget<()>> WidgetSequence for SingleWidget<W> {
    fn widgets(&mut self) -> Vec<&mut dyn FlexWidget> {
        vec![self]
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, env: &Env) {
        self.0.event(ctx, event, &mut (), env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, env: &Env) {
        self.0.lifecycle(ctx, event, &(), env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, env: &Env) {
        self.0.update(ctx, &(), env);
    }
}



pub struct WidgetTuple<
    WS0: WidgetSequence,
    WS1: WidgetSequence,
    WS2: WidgetSequence,
    WS3: WidgetSequence,
>(
    pub WS0,
    pub WS1,
    pub WS2,
    pub WS3,
);

impl<
    WS0: WidgetSequence,
    WS1: WidgetSequence,
    WS2: WidgetSequence,
    WS3: WidgetSequence,
> WidgetSequence for WidgetTuple<WS0, WS1, WS2, WS3> {
    fn widgets(&mut self) -> Vec<&mut dyn FlexWidget> {
        let mut all_widgets = Vec::new();
        all_widgets.append(&mut self.0.widgets());
        all_widgets.append(&mut self.1.widgets());
        all_widgets.append(&mut self.2.widgets());
        all_widgets.append(&mut self.3.widgets());
        all_widgets
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, env: &Env) {
        self.0.event(ctx, event, env);
        self.1.event(ctx, event, env);
        self.2.event(ctx, event, env);
        self.3.event(ctx, event, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, env: &Env) {
        self.0.lifecycle(ctx, event, env);
        self.1.lifecycle(ctx, event, env);
        self.2.lifecycle(ctx, event, env);
        self.3.lifecycle(ctx, event, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, env: &Env) {
        self.0.update(ctx, env);
        self.1.update(ctx, env);
        self.2.update(ctx, env);
        self.3.update(ctx, env);
    }
}


pub struct WidgetList<Child: WidgetSequence> {
    pub children: Vec<Child>,
}

impl<Child: WidgetSequence> WidgetSequence for WidgetList<Child> {
    fn widgets(&mut self) -> Vec<&mut dyn FlexWidget> {
        self.children.iter_mut().flat_map(|child| child.widgets()).collect()
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, env: &Env) {
        match event {
            _ => (),
        }
        for child in &mut self.children {
            child.event(ctx, event, env);
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, env: &Env) {
        for child in &mut self.children {
            child.lifecycle(ctx, event, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, env: &Env) {
        for child in &mut self.children {
            child.update(ctx, env);
        }
    }
}
