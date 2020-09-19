//! A description of a widget.

use std::any::Any;
use std::panic::Location;

use druid::widget;

use crate::any_widget::{Action, AnyWidget, DruidAppData};
use crate::cx::Cx;
use crate::id::Id;

pub trait View: AsAny + std::fmt::Debug {
    fn same(&self, other: &dyn View) -> bool;
    // This will yield Box<dyn Widget> in the future;
    fn make_widget(&self, id: Id) -> AnyWidget;
}

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
        cx.leaf_view(Box::new(self), Location::caller());
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
        let id = cx.leaf_view(Box::new(self), Location::caller());
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
    pub fn build(self, cx: &mut Cx) {
        cx.begin_view(Box::new(self), Location::caller())
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
        let row = crate::flex::Flex::row();
        AnyWidget::Flex(row)
    }
}
