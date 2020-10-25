//! A description of a widget.

use std::any::Any;
use std::panic::Location;

use druid::widget;

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
        AnyWidget::Label(widget::Label::new(self.0.to_string()))
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
        let button = widget::Button::new(self.0.clone())
            .on_click(move |_, data: &mut DruidAppData, _| data.queue_action(id, Action::Clicked));
        AnyWidget::Button(button)
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
        let row = crate::widget::Flex::row();
        AnyWidget::Flex(row)
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
        let column = crate::widget::Flex::column();
        AnyWidget::Flex(column)
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
        let text_box = crate::widget::TextBox::new(id, self.0.clone(), widget::TextBox::new());
        AnyWidget::TextBox(text_box)
    }
}

#[derive(Debug)]
pub struct Padding {
    pub(crate) insets: druid::Insets,
}

impl<I: Into<druid::Insets>> From<I> for Padding {
    fn from(insets: I) -> Self {
        Padding { insets: insets.into() }
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
        let row = crate::widget::Padding::new(self.insets);
        AnyWidget::Padding(row)
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
        let checkbox = crate::widget::Checkbox::new(id, self.state, self.label.clone());
        AnyWidget::Checkbox(checkbox)
    }
}

// TODO: this is commented out because the widget is not written yet.
/*
/// A wrapper for detecting click gestures.
#[derive(Debug)]
pub struct Clicked;

impl Clicked {
    pub fn new() -> Clicked {
        Clicked
    }

    #[track_caller]
    pub fn build<T>(self, cx: &mut Cx, f: impl FnOnce(&mut Cx)) -> bool {
        let id = cx.begin_view(Box::new(self), Location::caller());
        let result = f(cx);
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

    fn make_widget(&self, _id: Id) -> AnyWidget {
        let clicked = crate::widget::Clicked::new();
        AnyWidget::Clicked(clicked)
    }
}
*/
