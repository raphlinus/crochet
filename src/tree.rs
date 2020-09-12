//! A tree of render objects.

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
