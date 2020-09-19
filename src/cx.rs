//! The main Crochet interface.

use std::panic::Location;

use crate::tree::{MutCursor, Mutation, Tree};

pub struct Cx<'a> {
    mut_cursor: MutCursor<'a>,
}

impl<'a> Cx<'a> {
    /// Only public for experimentation.
    pub fn new(tree: &Tree) -> Cx {
        let mut_cursor = MutCursor::new(tree);
        Cx { mut_cursor }
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
}
