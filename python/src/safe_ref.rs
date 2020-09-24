//! A safe wrapper for borrowed mutable references.
//!
//! See: https://github.com/PyO3/pyo3/issues/1180

use pyo3::Python;
use std::sync::atomic::{AtomicPtr, Ordering};

/// A wrapper for a mutable reference.
pub struct SafeRef<T>(*mut AtomicPtr<T>);

unsafe impl<T> Send for SafeRef<T> {}

impl<T> Drop for SafeRef<T> {
    fn drop(&mut self) {
        unsafe {
            let box_ptr = self.0;
            let ptr = &*(box_ptr as *const AtomicPtr<String>);
            // CDSChecker suggests it's ok these are Relaxed
            if ptr.load(Ordering::Relaxed).is_null() {
                std::mem::drop(Box::from_raw(box_ptr));
            } else {
                ptr.store(std::ptr::null_mut(), Ordering::Relaxed);
            }
        }
    }
}

impl<T> SafeRef<T> {
    /// Run a closure with a wrapped reference.
    ///
    /// The `obj` reference is wrapped in a `SafeRef` so that it is cleared
    /// when the scope ends. In conjunction with the guarantees provided by
    /// running under the GIL, this should be entirely safe.
    ///
    /// Note: I've done some analysis and experimentation, but am not yet
    /// completely confident this actually is safe.
    pub fn scoped<'p, U>(_py: Python<'p>, obj: &mut T, f: impl FnOnce(SafeRef<T>) -> U) -> U {
        let box_ptr = Box::into_raw(Box::new(AtomicPtr::new(obj)));
        let wrapper = SafeRef(box_ptr);
        let result = f(wrapper);
        std::mem::drop(SafeRef(box_ptr));
        result
    }

    /// Get the mutable reference.
    ///
    /// The `Python<'p>` argument is a guarantee this is run under GIL.
    pub fn try_get_mut<'p>(&mut self, _py: Python<'p>) -> Option<&mut T> {
        unsafe {
            let ptr = (*self.0).load(Ordering::Relaxed);
            if ptr.is_null() {
                None
            } else {
                Some(&mut *ptr)
            }
        }
    }
}
