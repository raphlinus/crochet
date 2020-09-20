//! Types involving state.

use std::any::Any;
use std::fmt::Debug;

pub trait State {
    fn as_any(&self) -> &dyn Any;
    fn eq(&self, other: &dyn State) -> bool;
}

impl<T: PartialEq + 'static> State for T {
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
