//! The main Crochet interface.

use std::panic::Location;

use crate::any_widget::DruidAppData;
use crate::tree::{MutCursor, Mutation, Tree};

pub struct Cx<'a> {
    mut_cursor: MutCursor<'a>,
    app_data: &'a mut DruidAppData,
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

    #[track_caller]
    pub fn begin(&mut self, body: impl Into<String>) {
        self.mut_cursor.begin_loc(body.into(), Location::caller());
    }

    #[track_caller]
    pub fn leaf(&mut self, body: impl Into<String>) {
        self.mut_cursor.begin_loc(body.into(), Location::caller());
        self.mut_cursor.end();
    }

    pub fn end(&mut self) {
        self.mut_cursor.end();
    }

    #[track_caller]
    pub fn button(&mut self, label: impl AsRef<str>) -> bool {
        let body = format!("button: {}", label.as_ref());
        let id = self.mut_cursor.begin_loc(body, Location::caller());
        self.mut_cursor.end();
        self.app_data.dequeue_action(id).is_some()
    }

    #[track_caller]
    pub fn label(&mut self, label: impl AsRef<str>) {
        let body = format!("label: {}", label.as_ref());
        self.mut_cursor.begin_loc(body, Location::caller());
        self.mut_cursor.end();
    }
}
