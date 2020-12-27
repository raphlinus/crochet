//! The main Crochet interface.

use std::collections::HashMap;
use std::panic::Location;

// The unused annotations are mostly for optional async.
#[allow(unused)]
use druid::{ExtEventSink, SingleUse, Target};

#[cfg(feature = "async-std")]
use async_std::future::Future;

use crate::any_widget::{Action, DruidAppData};
#[cfg(feature = "async-std")]
use crate::app_holder::ASYNC;
use crate::id::Id;
use crate::state::State;
use crate::tree::{MutCursor, Mutation, Payload, Tree};
use crate::view::View;

pub struct Cx<'a> {
    mut_cursor: MutCursor<'a>,
    pub(crate) app_data: &'a mut DruidAppData,
    #[allow(unused)]
    resolved_futures: &'a HashMap<Id, Box<dyn State>>,
    #[allow(unused)]
    event_sink: &'a ExtEventSink,
}

impl<'a> Cx<'a> {
    /// Only public for experimentation.
    pub fn new(
        tree: &'a Tree,
        app_data: &'a mut DruidAppData,
        resolved_futures: &'a HashMap<Id, Box<dyn State>>,
        event_sink: &'a ExtEventSink,
    ) -> Cx<'a> {
        let mut_cursor = MutCursor::new(tree);
        Cx {
            mut_cursor,
            app_data,
            resolved_futures,
            event_sink,
        }
    }

    pub fn into_mutation(self) -> Mutation {
        self.mut_cursor.into_mutation()
    }

    pub fn end(&mut self) {
        self.mut_cursor.end_body();
    }

    /// Add a view as a leaf.
    ///
    /// This method is expected to be called mostly by the `build`
    /// methods on `View` implementors.
    pub fn leaf_view(&mut self, view: impl View + 'static, loc: &'static Location) -> Id {
        // Note: this always boxes (for convenience), but could be written
        // in terms of begin_core to avoid boxing on skip.
        let view = Box::new(view);
        let id = self.begin_view(view, loc);
        self.end();
        id
    }

    /// Begin a view element.
    ///
    /// This method is expected to be called mostly by the `build`
    /// methods on `View` implementors.
    ///
    /// The API may change to return a child cx.
    pub fn begin_view(&mut self, view: Box<dyn View>, loc: &'static Location) -> Id {
        let body = Payload::View(view);
        let is_new = self.begin_item_at(loc);
        let id = self.get_id();
        if is_new {
            self.set_payload(body);
        } else if &body != self.get_payload().unwrap() {
            self.set_payload(body);
        }
        self.end_item_and_begin_body();
        id
    }

    /// Traverse into a subtree only if the data has changed.
    ///
    /// The supplied callback *must* create only one widget. This is not
    /// enforced, and will probably be relaxed one way or other.
    ///
    /// This method also traverses into the subtree if any of its action
    /// queues are non-empty.
    #[track_caller]
    pub fn if_changed<T, U>(&mut self, data: T, f: impl FnOnce(&mut Cx) -> U) -> Option<U>
    where
        T: PartialEq + State + 'static,
    {
        let location = Location::caller();

        let mut changed = self.begin_item_at(location);
        if changed {
            self.set_payload(Payload::State(Box::new(data)));
        } else {
            if let Some(Payload::State(old_data)) = self.get_payload() {
                if let Some(old_data) = old_data.as_any().downcast_ref::<T>() {
                    if old_data != &data {
                        changed = true;
                        self.set_payload(Payload::State(Box::new(data)));
                    }
                } else {
                    panic!("State type of an if_changed block changed unexpectedly");
                }
            } else {
                panic!("Payload of if_canged block was not a Payload::State");
            }
        }
        self.end_item_and_begin_body();

        let actions = self.has_action();

        let result = if changed || actions {
            Some(f(self))
        } else {
            // TODO: here's a place that needs work if we relax the requirement
            // of exactly one child node.

            // Skip all children, by default they would be removed.
            self.mut_cursor.skip_one();
            None
        };

        self.end_body();
        result
    }

    /// Spawn a future when the data changes.
    ///
    /// When the data changes (including first insert), call `future_cb` and
    /// spawn the returned future. Note that if this callback needs to move
    /// the data into the async callback, it should clone it first.
    ///
    /// The value of the future is then made available to the main body
    /// callback.
    #[cfg(feature = "async-std")]
    #[track_caller]
    pub fn use_future<T, U, V, F, FC>(
        &mut self,
        data: &T,
        future_cb: FC,
        f: impl FnOnce(&mut Cx, Option<&U>) -> V,
    ) -> V
    where
        T: State + PartialEq + Clone + Sync + 'static,
        FC: FnOnce(&T) -> F,
        // Note: we can remove State bound
        U: Send + State + 'static,
        F: Future<Output = U> + Send + 'static,
    {
        let key = self.mut_cursor.key_from_loc(Location::caller());
        let (id, f_id, changed) = self.mut_cursor.begin_core(key, |id, old_body| {
            if let Some(Payload::Future(f_id, old_data)) = old_body {
                if let Some(old_data) = old_data.as_any().downcast_ref::<T>() {
                    if old_data == data {
                        (None, (id, *f_id, false))
                    } else {
                        // Types match, data not equal
                        let f_id = Id::new();
                        (
                            Some(Payload::Future(f_id, Box::new(data.clone()))),
                            (id, f_id, true),
                        )
                    }
                } else {
                    // Downcast failed; this shouldn't happen
                    let f_id = Id::new();
                    (
                        Some(Payload::Future(f_id, Box::new(data.clone()))),
                        (id, f_id, true),
                    )
                }
            } else {
                // Probably inserting new state
                let f_id = Id::new();
                (
                    Some(Payload::Future(f_id, Box::new(data.clone()))),
                    (id, f_id, true),
                )
            }
        });
        if changed {
            // Spawn the future.
            let future = future_cb(data);
            let sink = self.event_sink.clone();
            let boxed_future = Box::pin(async move {
                let result = future.await;
                let boxed_result: Box<dyn State> = Box::new(result);
                let payload = (id, f_id, boxed_result);
                if let Err(e) = sink.submit_command(ASYNC, SingleUse::new(payload), Target::Auto) {
                    println!("error {:?} submitting", e);
                }
            });
            async_std::task::spawn(boxed_future);
        }
        // Remove the "FutureResolved" action if it was sent.
        let _ = self.app_data.dequeue_action(id);
        let future_result = self
            .resolved_futures
            .get(&f_id)
            .and_then(|result| result.as_any().downcast_ref());
        let result = f(self, future_result);
        self.mut_cursor.end();
        result
    }

    /// A low-level method to skip nodes.
    ///
    /// There must be `n` nodes in the tree to skip.
    pub fn skip(&mut self, n: usize) {
        for _ in 0..n {
            self.mut_cursor.skip_one()
        }
    }

    /// A low-level method to delete nodes.
    ///
    /// There must be `n` nodes in the tree to delete.
    pub fn delete(&mut self, n: usize) {
        for _ in 0..n {
            self.mut_cursor.delete_one()
        }
    }

    /// A low-level method to insert a subtree.
    pub fn begin_insert(&mut self) {
        self.mut_cursor.begin_insert(Payload::Placeholder);
    }

    /// A low-level method to update a subtree.
    ///
    /// There must be an existing placeholder node at this location in the tree.
    pub fn begin_update(&mut self) {
        self.mut_cursor.begin_update(Payload::Placeholder);
    }

    /// Report whether the current element has an action.
    ///
    /// For the future, this should probably change to an `Option<usize>`,
    /// reporting how many elements to skip before the next action.
    pub fn has_action(&self) -> bool {
        self.mut_cursor
            .descendant_ids()
            .any(|id| self.app_data.has_action(id))
    }
}

impl<'a> Cx<'a> {
    #[track_caller]
    pub fn begin_item(&mut self) -> bool {
        self.mut_cursor.begin_item(Location::caller())
    }

    pub fn begin_item_at(&mut self, location: &'static Location) -> bool {
        self.mut_cursor.begin_item(location)
    }

    pub fn get_id(&self) -> Id {
        self.mut_cursor.get_current_id()
    }

    pub fn dequeue_action(&mut self) -> Option<Action> {
        self.app_data.dequeue_action(self.get_id())
    }

    pub fn get_payload(&self) -> Option<&Payload> {
        self.mut_cursor.get_current_payload()
    }

    pub fn set_payload(&mut self, payload: Payload) {
        self.mut_cursor.set_current_payload(payload);
    }

    pub fn end_item_and_begin_body(&mut self) {
        self.mut_cursor.end_item_and_begin_body();
    }

    pub fn end_body(&mut self) {
        self.mut_cursor.end_body();
    }
}
