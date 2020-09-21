//! Types involving state.

use std::any::Any;
use std::fmt::Debug;

/// An object suitable for storing as state.
///
/// We might separate the eq and Send roles. Further, we might have
/// one variant that supports PartialEq and another that supports
/// Druid Data.
pub trait State: Send {
    fn as_any(&self) -> &dyn Any;
    fn eq(&self, other: &dyn State) -> bool;
}

impl<T: PartialEq + Send + 'static> State for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq(&self, other: &dyn State) -> bool {
        if let Some(other) = other.as_any().downcast_ref() {
            self == other
        } else {
            false
        }
    }
}

impl Debug for dyn State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "state node of type {:?}", Any::type_id(self.as_any()))
    }
}
