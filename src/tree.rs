//! A tree of render objects.

use std::panic::Location;

use crate::key::{Caller, Key};

/// The type of an item in the tree.
///
/// This is a placeholder for experimentation.
#[derive(Debug, PartialEq)]
pub struct Item {
    key: Key,
    body: String,
}

/// A tree of items.
///
/// This is a somewhat unusual architecture. It is a list of
/// slots, where each can either push or pop a tree level.
#[derive(Default)]
pub struct Tree {
    slots: Vec<Slot>,
}

#[derive(Debug)]
pub enum Slot {
    Push(Item),
    Pop,
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


#[derive(Debug)]
pub struct Mutation(Vec<MutationItem>);

#[derive(Debug)]
pub enum MutationItem {
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

impl Tree {
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
    pub fn new() -> Mutation {
        Mutation(Vec::new())
    }

    pub fn skip(&mut self, n: usize) {
        if n > 0 {
            if let Some(MutationItem::Skip(old_n)) = self.0.last_mut() {
                *old_n += n;
            } else {
                self.0.push(MutationItem::Skip(n));
            }
        }
    }

    pub fn delete(&mut self, n: usize) {
        if n > 0 {
            if let Some(MutationItem::Delete(old_n)) = self.0.last_mut() {
                *old_n += n;
            } else {
                self.0.push(MutationItem::Delete(n));
            }
        }
    }

    pub fn insert(&mut self, new: Vec<Slot>) {
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
    pub fn insert_one(&mut self, slot: Slot) {
        // Just punt for now :)
        self.insert(vec![slot]);
    }

    pub fn update(&mut self, new: Vec<Slot>) {
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
    pub fn update_one(&mut self, slot: Slot) {
        // Just punt for now :)
        self.update(vec![slot]);
    }
}

impl<'a> MutCursor<'a> {
    pub fn new(tree: &Tree) -> MutCursor {
        MutCursor {
            tree,
            ix: 0,
            mutation: Mutation::new(),
            nest: 0,
            old_nest: 0,
        }
    }

    #[track_caller]
    pub fn begin(&mut self, body: String) {
        let caller = Location::caller().into();
        let key = Key::new(caller, self.seq_ix(caller));
        self.begin_item(Item { key, body });
    }

    #[track_caller]
    pub fn leaf(&mut self, body: String) {
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
                if let Some(Slot::Push(old)) = self.tree.slots.get(self.ix) {
                    if old.key == item.key {
                        self.ix += 1;
                        self.nest += 1;
                        self.old_nest += 1;
                        if old.body == item.body {
                            self.mutation.skip(1);
                        } else {
                            self.mutation.update_one(Slot::Push(item));
                        }
                        return;
                    }
                }
                }
        }
        self.nest += 1;
        self.mutation.insert_one(Slot::Push(item));
    }

    pub fn end(&mut self) {
        if self.nest == self.old_nest {
            let n_trim = self.count_trim();
            self.ix += n_trim + 1;
            self.mutation.delete(n_trim);
            self.mutation.skip(1);
        } else {
            self.nest -= 1;
            self.mutation.insert_one(Slot::Pop);
        }
    }

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
                Slot::Push(slot) => {
                    if nest == 0 && slot.key == key {
                        return Some(ix - self.ix);
                    }
                    nest += 1;
                }
                Slot::Pop => {
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
                Slot::Pop => nest += 1,
                Slot::Push(slot) => {
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

    /// The number of slots of the current node.
    #[allow(unused)]
    fn count_current(&self) -> usize {
        self.count_common(0)
    }

    /// The number of slots until the end of the current node.
    fn count_trim(&self) -> usize {
        self.count_common(1)
    }

    fn count_common(&self, mut nest: usize) -> usize {
        let mut ix = self.ix;
        loop {
            match self.tree.slots[ix] {
                Slot::Push(_) => nest += 1,
                Slot::Pop => {
                    nest -= 1;
                    if nest == 0 {
                        return ix - self.ix;
                    }
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
                Slot::Push(item) => {
                    println!("{}{:?}", "  ".repeat(nest), item);
                    nest += 1;
                }
                Slot::Pop => nest -= 1,
            }
        }
    }
}

/*
use std::panic::Location;
use std::sync::Arc;

use crate::key::Key;

/// A tree of render objects.
#[derive(Clone, Debug)]
pub struct Tree(pub Arc<Node>);

#[derive(Clone, Debug)]
pub struct Node {
    pub key: Key,
    /// The body of the node. This is a placeholder for a type
    /// appropriate to the UI task.
    pub body: String,
    pub children: Vec<Tree>,
    // TODO: stored state (for scope and memo)
    // TODO: action queue
}

impl Tree {
    /// Find the child with the specified key.
    ///
    /// This is a slow implementation but we're not worrying
    /// about that for now.
    fn find_key(&self, key: Key) -> Option<usize> {
        self.0.children.iter().position(|t| t.0.key == key)
    }

    fn get<'a>(&self, key_path: impl Iterator<Item = &'a Key>) -> Option<&Tree> {
        let mut node = self;
        for key in key_path {
            let child_ix = node.find_key(*key)?;
            node = &node.0.children[child_ix];
        }
        Some(node)
    }

    #[track_caller]
    pub fn new(root: impl Into<String>) -> Tree {
        Self::new_with_key(root.into(), Key::new(Location::caller(), 0))
    }

    pub(crate) fn new_with_key(body: String, key: Key) -> Tree {
        Tree(Arc::new(Node {
            key,
            body,
            children: Vec::new(),
        }))
    }
}
*/