//! The main Crochet interface.

use std::panic::Location;
use std::sync::Arc;

use crate::key::{Caller, Key};
use crate::Tree;

pub struct Cx {
    tree: Tree,
    cursor: Vec<usize>,
}

impl Cx {
    pub fn new(tree: Tree) -> Cx {
        Cx {
            tree,
            cursor: vec![0],
        }
    }

    pub fn into_tree(mut self) -> Tree {
        self.trim();
        self.tree
    }

    #[track_caller]
    pub fn begin(&mut self, body: impl Into<String>) {
        self.begin_raw(body.into(), Location::caller().into());
        self.cursor.push(0);
    }

    pub fn end(&mut self) {
        self.trim();
        self.cursor.pop();
        *self.cursor.last_mut().unwrap() += 1;
    }

    // Do most of the begin operation but don't update the cursor.
    fn begin_raw(&mut self, body: String, caller: Caller) {
        if let Some(tree) = self.cur_tree() {
            if tree.0.key.caller == caller {
                if self.seq_ix(caller) == Some(tree.0.key.seq_ix) {
                    if tree.0.body != body {
                        // Replace contents in place
                        Arc::make_mut(&mut self.cur_tree_mut().unwrap().0).body = body;
                    }
                    return;
                }
            }
        }
        let key = Key::new(caller, self.seq_ix(caller).unwrap());
        let pos = self.find_next(key);

        // Need to mutate, let's go.
        let mut node = &mut self.tree;
        for ix in &self.cursor[..self.cursor.len() - 1] {
            node = Arc::make_mut(&mut node.0).children.get_mut(*ix).unwrap();
        }
        let children = &mut Arc::make_mut(&mut node.0).children;
        let last_ix = *self.cursor.last().unwrap();

        if let Some(pos) = pos {
            // We found an item with matching key but not at the current
            // cursor position; delete elements up to it.
            children.drain(last_ix..pos);
            if children[last_ix].0.body != body {
                Arc::make_mut(&mut children[last_ix].0).body = body;
            }
        } else {
            // No matching key, insert at current cursor
            let new_node = Tree::new_with_key(body, key);
            children.insert(last_ix, new_node);
        }
    }

    fn trim(&mut self) {
        let mut node = &self.tree;
        for ix in &self.cursor[..self.cursor.len() - 1] {
            node = node.0.children.get(*ix).unwrap();
        }
        let last_ix = *self.cursor.last().unwrap();
        if last_ix != node.0.children.len() {
            let mut node = &mut self.tree;
            for ix in &self.cursor[..self.cursor.len() - 1] {
                node = Arc::make_mut(&mut node.0).children.get_mut(*ix).unwrap();
            }
            Arc::make_mut(&mut node.0).children.truncate(last_ix);
        }
    }

    /// The sequence number of the caller at the current cursor.
    fn seq_ix(&self, caller: Caller) -> Option<usize> {
        let mut node = &self.tree;
        for ix in &self.cursor[..self.cursor.len() - 1] {
            node = node.0.children.get(*ix)?;
        }
        let last_ix = *self.cursor.last()?;
        Some(
            node.0.children[..last_ix]
                .iter()
                .filter(|n| n.0.key.caller == caller)
                .count(),
        )
    }

    fn find_next(&self, key: Key) -> Option<usize> {
        let mut node = &self.tree;
        for ix in &self.cursor[..self.cursor.len() - 1] {
            node = node.0.children.get(*ix)?;
        }
        let last_ix = *self.cursor.last()?;
        node.0.children[last_ix..]
            .iter()
            .position(|n| n.0.key == key)
            .map(|pos| last_ix + pos)
    }

    /// The current tree node pointed to by the cursor.
    fn cur_tree(&self) -> Option<&Tree> {
        let mut node = &self.tree;
        for ix in &self.cursor {
            node = node.0.children.get(*ix)?;
        }
        Some(node)
    }

    fn cur_tree_mut(&mut self) -> Option<&mut Tree> {
        let mut node = &mut self.tree;
        for ix in &self.cursor {
            node = Arc::make_mut(&mut node.0).children.get_mut(*ix)?;
        }
        Some(node)
    }
}
