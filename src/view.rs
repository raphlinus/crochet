//! A description of a widget.

use std::panic::Location;
use std::{any::Any, f64::INFINITY};

use druid::{widget, UnitPoint};

use crate::any_widget::{Action, AnyWidget, DruidAppData};
use crate::cx::Cx;
use crate::id::Id;

pub trait View: AsAny + std::fmt::Debug {
    fn same(&self, other: &dyn View) -> bool;
    // This will yield Box<dyn Widget> in the future.
    fn make_widget(&self, id: Id) -> AnyWidget;
}

// Could use same AsAnyEqState trick to avoid repetitive eq impls.
pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

impl<T: 'static> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct Label(pub(crate) String);

impl Label {
    pub fn new(text: impl Into<String>) -> Label {
        Label(text.into())
    }

    #[track_caller]
    pub fn build(self, cx: &mut Cx) {
        cx.leaf_view(self, Location::caller());
    }
}

impl View for Label {
    fn same(&self, other: &dyn View) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.0 == other.0
        } else {
            false
        }
    }

    fn make_widget(&self, _id: Id) -> AnyWidget {
        let widget = widget::Label::new(self.0.to_string());
        AnyWidget::MutableWidget(Box::new(widget))
    }
}

#[derive(Debug)]
pub struct Button(pub(crate) String);

impl Button {
    pub fn new(text: impl Into<String>) -> Button {
        Button(text.into())
    }

    #[track_caller]
    pub fn build(self, cx: &mut Cx) -> bool {
        let id = cx.leaf_view(self, Location::caller());
        cx.app_data.dequeue_action(id).is_some()
    }
}

impl View for Button {
    fn same(&self, other: &dyn View) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.0 == other.0
        } else {
            false
        }
    }

    fn make_widget(&self, id: Id) -> AnyWidget {
        let widget = widget::Button::new(self.0.clone())
            .on_click(move |_, data: &mut DruidAppData, _| data.queue_action(id, Action::Clicked));
        AnyWidget::MutableWidget(Box::new(widget))
    }
}

#[derive(Debug)]
pub struct Row;

impl Row {
    pub fn new() -> Row {
        Row
    }

    #[track_caller]
    pub fn build<T>(self, cx: &mut Cx, f: impl FnOnce(&mut Cx) -> T) -> T {
        cx.begin_view(Box::new(self), Location::caller());
        let result = f(cx);
        cx.end();
        result
    }
}

impl View for Row {
    fn same(&self, other: &dyn View) -> bool {
        if let Some(_other) = other.as_any().downcast_ref::<Self>() {
            true
        } else {
            false
        }
    }

    fn make_widget(&self, _id: Id) -> AnyWidget {
        let widget = crate::widget::Flex::row();
        AnyWidget::MutableWidget(Box::new(widget))
    }
}

#[derive(Debug)]
pub struct Column;

impl Column {
    pub fn new() -> Column {
        Column
    }

    #[track_caller]
    pub fn build<T>(self, cx: &mut Cx, f: impl FnOnce(&mut Cx) -> T) -> T {
        cx.begin_view(Box::new(self), Location::caller());
        let result = f(cx);
        cx.end();
        result
    }
}

impl View for Column {
    fn same(&self, other: &dyn View) -> bool {
        if let Some(_other) = other.as_any().downcast_ref::<Self>() {
            true
        } else {
            false
        }
    }

    fn make_widget(&self, _id: Id) -> AnyWidget {
        let widget = crate::widget::Flex::column();
        AnyWidget::MutableWidget(Box::new(widget))
    }
}

#[derive(Debug)]
pub struct TextBox(pub(crate) String);

impl TextBox {
    pub fn new(content: impl Into<String>) -> Self {
        TextBox(content.into())
    }

    #[must_use]
    #[track_caller]
    pub fn build(self, cx: &mut Cx) -> Option<String> {
        let id = cx.leaf_view(self, Location::caller());
        cx.app_data.dequeue_action(id).map(|action| match action {
            Action::TextChanged(text) => text,
            _ => unreachable!("TextBox should never emit any Action other than TextChanged"),
        })
    }
}

impl View for TextBox {
    fn same(&self, other: &dyn View) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.0 == other.0
        } else {
            false
        }
    }

    fn make_widget(&self, id: Id) -> AnyWidget {
        let widget = crate::widget::TextBox::new(id, self.0.clone(), widget::TextBox::new());
        AnyWidget::MutableWidget(Box::new(widget))
    }
}

#[derive(Debug)]
pub struct Padding {
    pub(crate) insets: druid::Insets,
}

impl<I: Into<druid::Insets>> From<I> for Padding {
    fn from(insets: I) -> Self {
        Padding {
            insets: insets.into(),
        }
    }
}

impl Padding {
    pub fn new() -> Padding {
        Padding {
            insets: druid::Insets::ZERO,
        }
    }

    pub fn uniform(mut self, insets: f64) -> Self {
        self.insets = druid::Insets::uniform(insets);
        self
    }

    pub fn top(mut self, insets: f64) -> Self {
        self.insets.y0 = insets;
        self
    }

    #[track_caller]
    pub fn build<T>(self, cx: &mut Cx, f: impl FnOnce(&mut Cx) -> T) -> T {
        cx.begin_view(Box::new(self), Location::caller());
        let result = f(cx);
        cx.end();
        result
    }
}

impl View for Padding {
    fn same(&self, other: &dyn View) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.insets == other.insets
        } else {
            false
        }
    }

    fn make_widget(&self, _id: Id) -> AnyWidget {
        let widget = crate::widget::Padding::new(self.insets);
        AnyWidget::MutableWidget(Box::new(widget))
    }
}

#[derive(Debug, PartialEq)]
pub struct Checkbox {
    pub(crate) state: bool,
    pub(crate) label: String,
}

impl Checkbox {
    pub fn new(text: impl Into<String>, state: bool) -> Checkbox {
        Checkbox {
            state,
            label: text.into(),
        }
    }

    #[must_use]
    #[track_caller]
    pub fn build(self, cx: &mut Cx) -> bool {
        let old_state = self.state;
        let id = cx.leaf_view(self, Location::caller());
        cx.app_data
            .dequeue_action(id)
            .map(|action| match action {
                Action::Toggled(state) => state,
                _ => unreachable!("Checkbox should never emit any Action other than Toggled"),
            })
            .unwrap_or(old_state)
    }
}

impl View for Checkbox {
    fn same(&self, other: &dyn View) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }

    fn make_widget(&self, id: Id) -> AnyWidget {
        let widget = crate::widget::Checkbox::new(id, self.state, self.label.clone());
        AnyWidget::MutableWidget(Box::new(widget))
    }
}

/// A wrapper for detecting click gestures.
#[derive(Debug)]
pub struct Clicked;

impl Clicked {
    pub fn new() -> Clicked {
        Clicked
    }

    #[must_use]
    #[track_caller]
    pub fn build(self, cx: &mut Cx, f: impl FnOnce(&mut Cx)) -> bool {
        let id = cx.begin_view(Box::new(self), Location::caller());
        f(cx);
        cx.end();
        cx.app_data.dequeue_action(id).is_some()
    }
}

impl View for Clicked {
    fn same(&self, other: &dyn View) -> bool {
        if let Some(_other) = other.as_any().downcast_ref::<Self>() {
            true
        } else {
            false
        }
    }

    fn make_widget(&self, id: Id) -> AnyWidget {
        let widget = crate::widget::Click::new(id);
        AnyWidget::MutableWidget(Box::new(widget))
    }
}

/// A widget to do some custom painting.
///
/// # Important
/// This must always be wrapped in an `if_changed` block,
/// because it can not check whether any of the closures
/// parameters have changed.
#[derive(Clone)]
pub struct Painter<D> {
    pub(crate) data: D,
    pub(crate) paint: fn(&mut druid::PaintCtx, &druid::Env, data: &D),
}

impl<D> std::fmt::Debug for Painter<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Painter")
    }
}

impl<D: druid::Data> Painter<D> {
    pub fn new(data: D) -> Self {
        Painter {
            data,
            paint: |_, _, _| {},
        }
    }

    #[track_caller]
    pub fn build(mut self, cx: &mut Cx, paint: fn(&mut druid::PaintCtx, &druid::Env, &D)) {
        self.paint = paint;
        cx.leaf_view(self, Location::caller());
    }
}

impl<D: druid::Data> View for Painter<D> {
    fn same(&self, other: &dyn View) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.data.same(&other.data)
        } else {
            false
        }
    }

    fn make_widget(&self, _id: Id) -> AnyWidget {
        let widget = crate::widget::Painter::new(self.clone());
        AnyWidget::MutableWidget(Box::new(widget))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SizedBox {
    pub(crate) width: Option<f64>,
    pub(crate) height: Option<f64>,
}

impl<I: Into<druid::kurbo::Size>> From<I> for SizedBox {
    fn from(size: I) -> Self {
        let size = size.into();
        SizedBox {
            width: Some(size.width),
            height: Some(size.height),
        }
    }
}

impl SizedBox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn uniform(mut self, size: f64) -> Self {
        self.width = Some(size);
        self.height = Some(size);
        self
    }

    /// Set container's width.
    pub fn width(mut self, width: f64) -> Self {
        self.width = Some(width);
        self
    }

    /// Set container's height.
    pub fn height(mut self, height: f64) -> Self {
        self.height = Some(height);
        self
    }

    /// Expand container to fit the parent.
    ///
    /// Only call this method if you want your widget to occupy all available
    /// space. If you only care about expanding in one of width or height, use
    /// [`expand_width`] or [`expand_height`] instead.
    ///
    /// [`expand_height`]: #method.expand_height
    /// [`expand_width`]: #method.expand_width
    pub fn expand(mut self) -> Self {
        self.width = Some(INFINITY);
        self.height = Some(INFINITY);
        self
    }

    /// Expand the container on the x-axis.
    ///
    /// This will force the child to have maximum width.
    pub fn expand_width(mut self) -> Self {
        self.width = Some(INFINITY);
        self
    }

    /// Expand the container on the y-axis.
    ///
    /// This will force the child to have maximum height.
    pub fn expand_height(mut self) -> Self {
        self.height = Some(INFINITY);
        self
    }

    #[track_caller]
    pub fn build<T>(self, cx: &mut Cx, f: impl FnOnce(&mut Cx) -> T) -> T {
        cx.begin_view(Box::new(self), Location::caller());
        let result = f(cx);
        cx.end();
        result
    }
}

impl View for SizedBox {
    fn same(&self, other: &dyn View) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.width == other.width && self.height == other.height
        } else {
            false
        }
    }

    fn make_widget(&self, _id: Id) -> AnyWidget {
        let widget = crate::widget::SizedBox::new(&self);
        AnyWidget::MutableWidget(Box::new(widget))
    }
}

#[derive(Debug, Clone)]
pub struct Align {
    pub align: UnitPoint,
    pub width_factor: Option<f64>,
    pub height_factor: Option<f64>,
}

impl Align {
    /// Create widget with alignment.
    ///
    /// Note that the `align` parameter is specified as a `UnitPoint` in
    /// terms of left and right. This is inadequate for bidi-aware layout
    /// and thus the API will change when druid gains bidi capability.
    pub fn new(align: UnitPoint) -> Self {
        Align {
            align,
            width_factor: None,
            height_factor: None,
        }
    }

    /// Create centered widget.
    pub fn centered() -> Self {
        Align::new(UnitPoint::CENTER)
    }

    /// Create right-aligned widget.
    pub fn right() -> Self {
        Align::new(UnitPoint::RIGHT)
    }

    /// Create left-aligned widget.
    pub fn left() -> Self {
        Align::new(UnitPoint::LEFT)
    }

    /// Align only in the horizontal axis, keeping the child's size in the vertical.
    pub fn horizontal(align: UnitPoint) -> Self {
        Align {
            align,
            width_factor: None,
            height_factor: Some(1.0),
        }
    }

    /// Align only in the vertical axis, keeping the child's size in the horizontal.
    pub fn vertical(align: UnitPoint) -> Self {
        Align {
            align,
            width_factor: Some(1.0),
            height_factor: None,
        }
    }

    #[track_caller]
    pub fn build<T>(self, cx: &mut Cx, f: impl FnOnce(&mut Cx) -> T) -> T {
        cx.begin_view(Box::new(self), Location::caller());
        let result = f(cx);
        cx.end();
        result
    }
}

impl View for Align {
    fn same(&self, other: &dyn View) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.width_factor == other.width_factor
                && self.height_factor == other.height_factor
                && true // FIXME: && self.align == other.align
        } else {
            false
        }
    }

    fn make_widget(&self, _id: Id) -> AnyWidget {
        let widget = crate::widget::Align::new(&self);
        AnyWidget::MutableWidget(Box::new(widget))
    }
}
