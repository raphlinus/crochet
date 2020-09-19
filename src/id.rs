//! Unique identities.

use std::sync::atomic::{AtomicUsize, Ordering};

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// An identifier for an element.
///
/// It's a bit heavy-handed to have this id as well as widget
/// id in Druid; likely the two concepts should be unified. But
#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct Id(usize);

impl Id {
    /// Allocate a new unique id.
    pub fn new() -> Id {
        Id(ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}
