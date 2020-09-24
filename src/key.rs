//! Unique keys and key paths.

use std::hash::Hash;
use std::panic::Location;

/// A unique call location.
///
/// These come from `#[track_caller]` annotations. It is a newtype
/// so we can use it as a key in various contexts; the traits we
/// want are not implemented on the inner type.
#[derive(Clone, Copy, Debug)]
pub struct Caller(&'static Location<'static>);

#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Debug)]
pub struct Key {
    /// The caller that originated the mutation.
    pub(crate) caller: Caller,
    /// The sequence index.
    ///
    /// At some point, we probably should accommodate user-provided
    /// stable identities, but for now we just assume that it consists
    /// of the caller and sequence number.
    pub(crate) seq_ix: usize,
}

impl Key {
    pub fn new(caller: impl Into<Caller>, seq_ix: usize) -> Key {
        Key {
            caller: caller.into(),
            seq_ix,
        }
    }

    /// A null key, which will always equal itself.
    ///
    /// In the future, this might be implemented differently, as Key will
    /// possibly expand to accommodate user-provided keys and callers from
    /// different runtimes such as scripting languages.
    pub fn null() -> Key {
        #[track_caller]
        fn null_caller() -> Caller {
            Location::caller().into()
        }
        Key::new(null_caller(), 0)
    }
}

impl Caller {
    /// The pointer to the location metadata
    ///
    /// Unique locations are expected to have unique pointers. This
    /// is perhaps not formally guaranteed by the language spec, but
    /// it's hard to imagine how it can be implemented otherwise.
    fn as_ptr(&self) -> *const Location<'static> {
        self.0
    }
}

impl PartialEq for Caller {
    fn eq(&self, other: &Caller) -> bool {
        self.as_ptr() == other.as_ptr()
    }
}

impl Eq for Caller {}

impl Hash for Caller {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_ptr().hash(state)
    }
}

impl PartialOrd for Caller {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_ptr().partial_cmp(&other.as_ptr())
    }
}

impl Ord for Caller {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_ptr().cmp(&other.as_ptr())
    }
}

impl From<&'static Location<'static>> for Caller {
    fn from(inner: &'static Location<'static>) -> Self {
        Caller(inner)
    }
}
