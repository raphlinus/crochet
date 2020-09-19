//! The main Crochet interface.

use std::panic::Location;

use crate::any_widget::DruidAppData;
use crate::id::Id;
use crate::tree::{MutCursor, Mutation, Payload, Tree};
use crate::view::View;

pub struct Cx<'a> {
    mut_cursor: MutCursor<'a>,
    pub(crate) app_data: &'a mut DruidAppData,
}

impl<'a> Cx<'a> {
    /// Only public for experimentation.
    pub fn new(tree: &'a Tree, app_data: &'a mut DruidAppData) -> Cx<'a> {
        let mut_cursor = MutCursor::new(tree);
        Cx {
            mut_cursor,
            app_data,
        }
    }

    pub fn into_mutation(self) -> Mutation {
        self.mut_cursor.into_mutation()
    }

    pub fn end(&mut self) {
        self.mut_cursor.end();
    }

    /// Add a view as a leaf.
    ///
    /// This method is expected to be called mostly by the `build`
    /// methods on `View` implementors.
    pub fn leaf_view(&mut self, view: Box<dyn View>, loc: &'static Location) -> Id {
        let body = Payload::View(view);
        let id = self.mut_cursor.begin_loc(body, loc);
        self.mut_cursor.end();
        id
    }

    /// Begin a view element.
    ///
    /// This method is expected to be called mostly by the `build`
    /// methods on `View` implementors.
    ///
    /// The API will change to return a child cx.
    pub fn begin_view(&mut self, view: Box<dyn View>, loc: &'static Location) {
        let body = Payload::View(view);
        self.mut_cursor.begin_loc(body, loc);
    }
}
