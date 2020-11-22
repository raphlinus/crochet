//! This module contains widgets that have been cut and pasted from Druid,
//! adapted to support the "mutate" method.

mod single;
pub use single::SingleChild;

mod flex;
pub use flex::Flex;

mod textbox;
pub use textbox::TextBox;

mod padding;
pub use padding::Padding;

mod checkbox;
pub use checkbox::Checkbox;

mod click;
pub use click::Click;

mod sized_box;
pub use sized_box::SizedBox;
