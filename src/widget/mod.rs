//! This module contains widgets that have been cut and pasted from Druid,
//! adapted to support the "mutate" method.

mod flex;
pub use flex::Flex;

mod textbox;
pub use textbox::TextBox;

mod padding;
pub use padding::Padding;

mod checkbox;
pub use checkbox::Checkbox;

// TODO: want a "clicked" wrapper.
