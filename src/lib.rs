//! An exploration into reactive UI
//!
//! This library is an alternative interface on top of Druid for
//! applications to build and manipulate UI. It is a prototype, for
//! the focused goal of exploring a new architecture.
//!
//! As a prototype, there are compromises. Since it is not easy to
//! add a new method to Druid's `Widget` trait (largely because Rust
//! doesn't support object-oriented paradigms well), it achieves the
//! goal of adding a tree mutation method through the `AnyWidget`
//! enum. That design is not intended for long-term use. Also, the
//! tree mutation data structures and algorithms are all designed in
//! a straightforward way, not designed to be efficient at scale.

mod any_widget;
mod app_holder;
mod cx;
mod flex;
mod id;
mod key;
mod state;
mod tree;
mod view;

pub use any_widget::DruidAppData;
pub use app_holder::AppHolder;
pub use cx::Cx;
pub use id::Id;
pub use state::State;
pub use tree::{MutCursor, MutIterItem, Mutation, MutationIter, Payload, Tree};
pub use view::{Button, Column, Label, Row};
