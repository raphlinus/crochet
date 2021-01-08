//! A list component.

use crate::id::Id;
use crate::view::Column;
use crate::Cx;

/// A vector that tracks modifications.
///
/// This is a simple wrapper around vec that adds a stable id,
/// for tracking insertions and deletions, and a revision id,
/// for tracking updates.
///
/// It's designed in such a way that the same data could be used
/// used in multiple list views, but that might be overengineering.
/// A simpler approach might be based on dirty tracking.
///
/// Obviously right now the implementation is super-simple and
/// not efficient. But the idea is that a similar interface might
/// support some fancy incremental implementation.
pub struct ListData<T>(Vec<ListItem<T>>);

impl<T> Default for ListData<T> {
    fn default() -> Self {
        ListData(Vec::new())
    }
}

struct ListItem<T> {
    stable_id: Id,
    rev_id: Id,
    val: T,
}

/// A list view component.
#[derive(Default)]
pub struct List {
    selected: Option<Id>,
    old_selected: Option<Id>,
    items: Vec<(Id, Id)>,
}

impl<T> ListItem<T> {
    fn new(val: T) -> Self {
        let id = Id::new();
        // There is the convention that rev_id == stable_id for a
        // newly created (never modified) item, but maybe we don't
        // rely on it.
        ListItem {
            stable_id: id,
            rev_id: id,
            val,
        }
    }
}

impl<T> ListData<T> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, val: T) {
        self.0.push(ListItem::new(val));
    }

    pub fn insert_at_ix(&mut self, ix: usize, val: T) {
        self.0.insert(ix, ListItem::new(val));
    }

    pub fn remove_at_ix(&mut self, ix: usize) {
        self.0.remove(ix);
    }

    pub fn set_at_ix(&mut self, ix: usize, val: T) {
        let item = &mut self.0[ix];
        item.rev_id = Id::new();
        item.val = val;
    }

    pub fn stable_id_at_ix(&self, ix: usize) -> Id {
        self.0[ix].stable_id
    }

    pub fn rev_id_at_ix(&self, ix: usize) -> Id {
        self.0[ix].rev_id
    }

    pub fn get_at_ix(&mut self, ix: usize) -> &T {
        &self.0[ix].val
    }

    pub fn find_id(&self, id: Id) -> Option<usize> {
        // A scalable version would not be O(n) here
        self.0.iter().position(|item| id == item.stable_id)
    }

    pub fn swap(&mut self, ix_a: usize, ix_b: usize) {
        self.0.swap(ix_a, ix_b);
    }
}

impl List {
    /// Update the view tree to reflect changes in the list.
    ///
    /// Call the supplied callback for every new or updated item.
    pub fn run<T, F>(&mut self, cx: &mut Cx, data: &ListData<T>, mut item_cb: F)
    where
        F: FnMut(&mut Cx, bool, Id, &T),
    {
        // TODO: track caller, pass to the column; or just do a tree around it.
        Column::new().build(cx, |cx| {
            let mut old_ix = 0;
            let mut new_items = Vec::new();
            for item in &data.0 {
                let id = item.stable_id;
                let is_selected = self.selected == Some(id);
                let was_selected = self.old_selected == Some(id);
                let update_sel = is_selected != was_selected;
                if let Some(pos) = self.items[old_ix..].iter().position(|(s, _r)| *s == id) {
                    // Item with this id found; update if needed.
                    cx.delete(pos);
                    old_ix += pos;
                    let has_action = cx.has_action();
                    if self.items[old_ix].1 != item.rev_id || update_sel || has_action {
                        cx.begin_update();
                        item_cb(cx, is_selected, id, &item.val);
                        cx.end();
                        self.items[old_ix].1 = item.rev_id;
                    } else {
                        cx.skip(1);
                    }
                    old_ix += 1;
                } else {
                    // No item found with this id; insert.
                    cx.begin_insert();
                    item_cb(cx, is_selected, id, &item.val);
                    cx.end();
                }
                new_items.push((item.stable_id, item.rev_id));
            }
            cx.delete(self.items.len() - old_ix);
            self.items = new_items;
        });
        self.old_selected = self.selected();
    }

    pub fn select(&mut self, id: impl Into<Option<Id>>) {
        self.selected = id.into();
    }

    /// The current selected widget, if any.
    pub fn selected(&self) -> Option<Id> {
        self.selected
    }
}
