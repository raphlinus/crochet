//! The main Crochet interface.

use std::collections::HashMap;
use std::panic::Location;

// The unused annotations are mostly for optional async.
#[allow(unused)]
use druid::{ExtEventSink, SingleUse, Target};

#[cfg(feature = "async-std")]
use async_std::future::Future;

use crate::any_widget::DruidAppData;
#[cfg(feature="async-std")]
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
        let actions = self
            .mut_cursor
            .descendant_ids()
            .any(|id| self.app_data.has_action(id));
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

    /// Spawn a future when the data changes.
    ///
    /// When the data changes (including first insert), call `future_cb` and
    /// spawn the returned future.
    ///
    /// The value of the future is then made available to the main body
    /// callback.
    #[cfg(feature = "async-std")]
    #[track_caller]
    pub fn use_future<T, U, V, F, FC>(
        &mut self,
        data: T,
        future_cb: FC,
        f: impl FnOnce(&mut Cx, Option<&U>) -> V,
    ) -> V
    where
        T: State + PartialEq + Clone + 'static,
        FC: FnOnce(&T) -> F,
        // Note: we can remove State bound
        U: Send + State + 'static,
        F: Future<Output = U> + Send + 'static,
    {
        let key = self.mut_cursor.key_from_loc(Location::caller());
        // Note: ideally we want to remove this clone, but it's tricky.
        // After the `begin_core`, either there was no change, in which
        // case two potentially usable copies, the `data` argument and
        // the copy in the tree (which are equal), or there was a change,
        // in which case it was moved into the mutation.
        //
        // So I think it's possible, but will require very advanced
        // fighting with the borrow checker, possibly even unsafe. To
        // avoid dealing with that now, just clone.
        let data_clone = data.clone();
        let (id, f_id, changed) = self.mut_cursor.begin_core(key, |id, old_body| {
            if let Some(Payload::Future(f_id, old_data)) = old_body {
                if let Some(old_data) = old_data.as_any().downcast_ref::<T>() {
                    if old_data == &data {
                        (None, (id, *f_id, false))
                    } else {
                        // Types match, data not equal
                        let f_id = Id::new();
                        (Some(Payload::Future(f_id, Box::new(data))), (id, f_id, true))
                    }
                } else {
                    // Downcast failed; this shouldn't happen
                    let f_id = Id::new();
                    (Some(Payload::Future(f_id, Box::new(data))), (id, f_id, true))
                }
            } else {
                // Probably inserting new state
                let f_id = Id::new();
                (Some(Payload::Future(f_id, Box::new(data))), (id, f_id, true))
            }
        });
        if changed {
            // Spawn the future.
            let future = future_cb(&data_clone);
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
}
