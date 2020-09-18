//! A tree of render objects.

use std::panic::Location;

use crate::key::{Caller, Key};

/// The payload of an item in the tree.
///
/// This is a string for now, just for experimentation.
/// It will shortly become an enum containing state nodes,
/// descriptors for creating render elements, and possibly
/// other things.
pub type Payload = String;

/// The type of an item in the tree.
#[derive(Debug, PartialEq)]
pub struct Item {
    key: Key,
    body: Payload,
}

/// A tree of items.
///
/// Conceptually, a tree follows this grammar:
///
/// ```
/// tree := element*
/// element := begin attribute* tree end
/// ```
///
/// Attributes are TODO.
///
/// Each "begin" item carries a payload. It is also associated with
/// a key. Currently keys are derived from caller and sequence number,
/// but we will support user-provided keys also.
///
/// In implementation, this is a somewhat unusual architecture. It is a
/// list of slots, where each can either push or pop a tree level. But
/// ideally implementation details are hidden, and this type can be
/// considered an abstract interface.
#[derive(Default)]
pub struct Tree {
    slots: Vec<Slot>,
}

#[derive(Debug)]
pub enum Slot {
    Begin(Item),
    End,
}

pub struct MutCursor<'a> {
    tree: &'a Tree,
    ix: usize,
    mutation: Mutation,
    // Current nesting level (mutating)
    nest: usize,
    // Nesting level in old tree
    old_nest: usize,
}

/// A tree mutation.
///
/// A mutation is description of the delta from the old state of the tree
/// to the new. In this architecture, we don't mutate the tree in place,
/// but rather produce an explicit mutation object, which is then applied
/// after the app logic runs.
///
/// Note that there are some soundness invariants on a mutation. In addition
/// to the new tree being balanced:
///
/// * The sum of Skip, Delete, and Update sizes equals the number of slots
/// in the original tree.
/// * If a `Begin` is in a `Delete`, the delete covers all the way to the
/// matching `End`.
/// * If a `Begin` is in an `Insert`, the insert covers all the way to the
/// matching `End`.
/// * `Update` doesn't change the `Slot` variant.
#[derive(Debug)]
pub struct Mutation(Vec<MutationItem>);

/// One item in the internal representation of a tree mutation.
#[derive(Debug)]
enum MutationItem {
    /// No change for the next n slots.
    Skip(usize),
    /// Delete the next n slots.
    Delete(usize),
    /// Insert new items at the current location.
    Insert(Vec<Slot>),
    /// Update existing items.
    ///
    /// Update is similar to delete + insert, but is intended to
    /// preserve the identity of those tree locations.
    Update(Vec<Slot>),
}

/// One item in a mutation for a single node.
pub enum MutIterItem<'a> {
    /// No change for the next n children.
    Skip(usize),
    /// Delete the next n children.
    Delete(usize),
    /// Insert a new child.
    Insert(&'a Payload, MutationIter<'a>),
    /// Update the child.
    ///
    /// For discussion: include old + new values?
    Update(Option<&'a Payload>, MutationIter<'a>),
}

#[derive(Clone)]
/// An iterator for reading out a tree mutation.
pub struct MutationIter<'a> {
    tree: &'a Tree,
    mutation: &'a [MutationItem],
    /// An index to the slot number in the tree.
    tree_ix: usize,
    /// An index to the item in the mutation.
    ///
    /// Discussion: might be better to just reduce
    /// the mutation slice.
    mut_ix: usize,
    /// The number of slots already consumed in the currently
    /// open mutation item.
    consumed: usize,
}

impl Tree {
    /// Apply the mutation, mutating the tree.
    pub fn mutate(&mut self, mutation: Mutation) {
        // This implementation isn't trying to be efficient.
        let mut ix = 0;
        for mut_item in mutation.0 {
            match mut_item {
                MutationItem::Skip(n) => ix += n,
                MutationItem::Delete(n) => {
                    self.slots.drain(ix..ix + n);
                }
                MutationItem::Insert(new) => {
                    let n = new.len();
                    self.slots.splice(ix..ix, new);
                    ix += n;
                }
                MutationItem::Update(new) => {
                    let n = new.len();
                    self.slots.splice(ix..ix + n, new);
                    ix += n;
                }
            }
        }
    }
}

impl Mutation {
    fn new() -> Mutation {
        Mutation(Vec::new())
    }

    fn skip(&mut self, n: usize) {
        if n > 0 {
            if let Some(MutationItem::Skip(old_n)) = self.0.last_mut() {
                *old_n += n;
            } else {
                self.0.push(MutationItem::Skip(n));
            }
        }
    }

    fn delete(&mut self, n: usize) {
        if n > 0 {
            if let Some(MutationItem::Delete(old_n)) = self.0.last_mut() {
                *old_n += n;
            } else {
                self.0.push(MutationItem::Delete(n));
            }
        }
    }

    fn insert(&mut self, new: Vec<Slot>) {
        if !new.is_empty() {
            if let Some(MutationItem::Insert(old)) = self.0.last_mut() {
                old.extend(new);
            } else {
                self.0.push(MutationItem::Insert(new));
            }
        }
    }

    /// Insert a single slot.
    ///
    /// This is semantically the same as insert, but potentially more
    /// efficient, and also convenient.
    fn insert_one(&mut self, slot: Slot) {
        // Just punt for now :)
        self.insert(vec![slot]);
    }

    fn update(&mut self, new: Vec<Slot>) {
        if !new.is_empty() {
            if let Some(MutationItem::Update(old)) = self.0.last_mut() {
                old.extend(new);
            } else {
                self.0.push(MutationItem::Update(new));
            }
        }
    }

    /// Update a single slot.
    ///
    /// This is semantically the same as update, but potentially more
    /// efficient, and also convenient.
    fn update_one(&mut self, slot: Slot) {
        // Just punt for now :)
        self.update(vec![slot]);
    }
}

impl<'a> MutCursor<'a> {
    /// Start building a tree mutation.
    pub fn new(tree: &Tree) -> MutCursor {
        MutCursor {
            tree,
            ix: 0,
            mutation: Mutation::new(),
            nest: 0,
            old_nest: 0,
        }
    }

    /// Begin an element.
    #[track_caller]
    pub fn begin(&mut self, body: Payload) {
        let caller = Location::caller().into();
        let key = Key::new(caller, self.seq_ix(caller));
        self.begin_item(Item { key, body });
    }

    /// Add a leaf element.
    #[track_caller]
    pub fn leaf(&mut self, body: Payload) {
        let caller = Location::caller().into();
        let key = Key::new(caller, self.seq_ix(caller));
        self.begin_item(Item { key, body });
        self.end();
    }

    fn begin_item(&mut self, item: Item) {
        if self.nest == self.old_nest {
            // TODO: really should have fast path if the key matches
            if let Some(n) = self.find_key(item.key) {
                self.ix += n;
                self.mutation.delete(n);
                if let Some(Slot::Begin(old)) = self.tree.slots.get(self.ix) {
                    if old.key == item.key {
                        self.ix += 1;
                        self.nest += 1;
                        self.old_nest += 1;
                        if old.body == item.body {
                            self.mutation.skip(1);
                        } else {
                            self.mutation.update_one(Slot::Begin(item));
                        }
                        return;
                    }
                }
            }
        }
        self.nest += 1;
        self.mutation.insert_one(Slot::Begin(item));
    }

    /// End an element.
    pub fn end(&mut self) {
        if self.nest == self.old_nest {
            let n_trim = self.count_trim();
            self.ix += n_trim + 1;
            self.mutation.delete(n_trim);
            self.mutation.skip(1);
        } else {
            self.nest -= 1;
            self.mutation.insert_one(Slot::End);
        }
    }

    /// Reap the mutation.
    pub fn into_mutation(self) -> Mutation {
        self.mutation
    }

    /// Find the key in the current node.
    ///
    /// Returns number of slots until the key.
    fn find_key(&self, key: Key) -> Option<usize> {
        let mut nest = 0;
        let mut ix = self.ix;
        while ix < self.tree.slots.len() {
            match &self.tree.slots[ix] {
                Slot::Begin(slot) => {
                    if nest == 0 && slot.key == key {
                        return Some(ix - self.ix);
                    }
                    nest += 1;
                }
                Slot::End => {
                    if nest == 0 {
                        return None;
                    }
                    nest -= 1;
                }
            }
            ix += 1;
        }
        None
    }

    /// The number of previous items in this node with this caller.
    fn seq_ix(&self, caller: Caller) -> usize {
        let mut seq_ix = 0;
        let mut nest = 0;
        let mut ix = self.ix;
        while ix > 0 {
            ix -= 1;
            match &self.tree.slots[ix] {
                Slot::End => nest += 1,
                Slot::Begin(slot) => {
                    if nest == 0 {
                        break;
                    } else if nest == 1 && slot.key.caller == caller {
                        seq_ix += 1;
                    }
                    nest -= 1;
                }
            }
        }
        seq_ix
    }

    /// The number of slots until the end of the current node.
    fn count_trim(&self) -> usize {
        let mut nest = 0usize;
        let mut ix = self.ix;
        loop {
            match self.tree.slots[ix] {
                Slot::Begin(_) => nest += 1,
                Slot::End => {
                    if nest == 0 {
                        return ix - self.ix;
                    }
                    nest -= 1;
                }
            }
            ix += 1;
        }
    }
}

impl Tree {
    pub fn dump(&self) {
        let mut nest = 0;
        for slot in &self.slots {
            match slot {
                Slot::Begin(item) => {
                    println!("{}{:?}", "  ".repeat(nest), item);
                    nest += 1;
                }
                Slot::End => nest -= 1,
            }
        }
    }

    /// The number of slots taken by the element starting at `ix`.
    fn count_slots(&self, ix: usize) -> Option<usize> {
        if let Slot::End = self.slots[ix] {
            None
        } else {
            Some(count_slots(&self.slots[ix..]))
        }
    }
}

/// The number of slots taken by the element at the beginning of the slice.
fn count_slots(slots: &[Slot]) -> usize {
    let mut ix = 0;
    let mut nest = 0usize;
    loop {
        match slots[ix] {
            Slot::Begin(_) => nest += 1,
            Slot::End => {
                nest -= 1;
                if nest == 0 {
                    return ix + 1;
                }
            }
        }
        ix += 1;
    }
}

impl<'a> Iterator for MutationIter<'a> {
    type Item = MutIterItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        //println!("next tree_ix={} mut_ix={} consumed={}", self.tree_ix, self.mut_ix, self.consumed);
        if let Some(cur) = self.mutation.get(self.mut_ix) {
            match cur {
                MutationItem::Skip(n) => {
                    let cur_slots = self.tree.count_slots(self.tree_ix)?;
                    if cur_slots + self.consumed <= *n {
                        // Skip this element.
                        self.advance(cur_slots);
                        // TODO: get fancier about aggregating.
                        Some(MutIterItem::Skip(1))
                    } else {
                        // Element contains a mutation; descend.
                        let mut child_iter = self.clone();
                        child_iter.advance(1);
                        self.advance(cur_slots);
                        Some(MutIterItem::Update(None, child_iter))
                    }
                }
                MutationItem::Delete(_) => {
                    let cur_slots = self.tree.count_slots(self.tree_ix)?;
                    self.advance(cur_slots);
                    Some(MutIterItem::Delete(1))
                }
                MutationItem::Insert(slots) => {
                    if let Slot::Begin(item) = &slots[self.consumed] {
                        let mut child_iter = self.clone();
                        child_iter.consumed += 1;
                        let cur_slots = count_slots(&slots[self.consumed..]);
                        self.consumed += cur_slots;
                        if self.consumed == slots.len() {
                            self.mut_ix += 1;
                            self.consumed = 0;
                        }
                        Some(MutIterItem::Insert(&item.body, child_iter))
                    } else {
                        None
                    }
                }
                MutationItem::Update(slots) => {
                    if let Slot::Begin(item) = &slots[self.consumed] {
                        let mut child_iter = self.clone();
                        child_iter.advance(1);
                        let cur_slots = self.tree.count_slots(self.tree_ix)?;
                        self.advance(cur_slots);
                        Some(MutIterItem::Update(Some(&item.body), child_iter))
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        }
    }
}

impl<'a> MutationIter<'a> {
    /// Start an iteration over a mutation.
    pub fn new(tree: &'a Tree, mutation: &'a Mutation) -> MutationIter<'a> {
        MutationIter {
            tree,
            mutation: &mutation.0,
            tree_ix: 0,
            mut_ix: 0,
            consumed: 0,
        }
    }

    /// Advance the iterator forward by `adv` slots.
    ///
    /// The number of slots is measured relative to the original tree.
    fn advance(&mut self, mut adv: usize) {
        //println!("advance {}, tree_ix={}, mut_ix={}, consumed={}", adv, self.tree_ix, self.mut_ix, self.consumed);
        self.tree_ix += adv;
        while adv > 0 {
            let cur = &self.mutation[self.mut_ix];
            match cur {
                MutationItem::Skip(n) | MutationItem::Delete(n) => {
                    if n - self.consumed <= adv {
                        adv -= n - self.consumed;
                        self.mut_ix += 1;
                        self.consumed = 0;
                    } else {
                        self.consumed += adv;
                        break;
                    }
                }
                MutationItem::Insert(_) => self.mut_ix += 1,
                MutationItem::Update(slots) => {
                    let n = slots.len();
                    if n - self.consumed <= adv {
                        adv -= n - self.consumed;
                        self.mut_ix += 1;
                        self.consumed = 0;
                    } else {
                        self.consumed += adv;
                        break;
                    }
                }
            }
        }
    }
}
