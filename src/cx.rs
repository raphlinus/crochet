//! The main Crochet interface.

use std::panic::Location;

use crate::any_widget::DruidAppData;
use crate::id::Id;
use crate::state::State;
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
    pub fn leaf_view(&mut self, view: impl View + 'static, loc: &'static Location) -> Id {
        // Note: this always boxes (for convenience), but could be written
        // in terms of begin_core to avoid boxing on skip.
        let view = Box::new(view);
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

    /// Traverse into a subtree only if the data has changed.
    ///
    /// The supplied callback *must* create only one widget. This is not
    /// enforced, and will probably be relaxed one way or other.
    ///
    /// This method also traverses into the subtree if any of its action
    /// queues are non-empty.
    #[track_caller]
    pub fn if_changed<T: PartialEq + State + 'static, U>(
        &mut self,
        data: T,
        f: impl FnOnce(&mut Cx) -> U,
    ) -> Option<U> {
        let key = self.mut_cursor.key_from_loc(Location::caller());
        let changed = self.mut_cursor.begin_core(key, |_id, old_body| {
            if let Some(Payload::State(old_data)) = old_body {
                if let Some(old_data) = old_data.as_any().downcast_ref::<T>() {
                    if old_data == &data {
                        (None, false)
                    } else {
                        // Types match, data not equal
                        (Some(Payload::State(Box::new(data))), true)
                    }
                } else {
                    // Downcast failed; this shouldn't happen
                    (Some(Payload::State(Box::new(data))), true)
                }
            } else {
                // Probably inserting new state
                (Some(Payload::State(Box::new(data))), true)
            }
        });
        let actions = self.mut_cursor.descendant_ids().any(|id| self.app_data.has_action(id));
        let result = if changed || actions {
            Some(f(self))
        } else {
            // TODO: here's a place that needs work if we relax the requirement
            // of exactly one child node.
            self.mut_cursor.skip_one();
            None
        };
        self.mut_cursor.end();
        result
    }
}
