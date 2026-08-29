//! A DAG representing the hierarchy within a scene.

use crate::scene::{EntityId, Scene};
use std::collections::{BTreeSet, HashMap};

#[derive(Default)]
pub struct Hierarchy {
    children: HashMap<EntityId, BTreeSet<EntityId>>,
    parents: HashMap<EntityId, EntityId>,
}

impl Hierarchy {
    /// Get the parent of a given entity.
    pub fn parent(&self, child: EntityId) -> Option<EntityId> {
        self.parents.get(&child).copied()
    }

    /// Returns true iff this entity has a parent entity.
    pub fn has_parent(&self, entity: EntityId) -> bool {
        self.parents.contains_key(&entity)
    }

    pub fn is_parent(&self, entity: EntityId, child: EntityId) -> bool {
        self.parents.get(&child) == Some(&entity)
    }

    /// Returns true iff `entity` is an ancestor of `child`.
    pub fn is_ancestor(&self, entity: EntityId, child: EntityId) -> bool {
        let mut current = child;

        while let Some(&parent) = self.parents.get(&current) {
            if current == entity {
                return true;
            }

            current = parent;
        }

        false
    }

    /// Get all children for a given entity.
    pub fn children(&self, parent: EntityId) -> Vec<EntityId> {
        self.children
            .get(&parent)
            .map(|x| x.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Returns true iff this entity has children attached.
    pub fn has_children(&self, entity: EntityId) -> bool {
        self.children.get(&entity).is_some_and(|x| !x.is_empty())
    }

    /// Remove parent from child if it has one.
    pub fn graft(&mut self, child: EntityId) {
        // Only do something if we actually have a parent
        let Some(parent) = self.parents.remove(&child) else {
            return;
        };

        // Remove the child from the parents array.
        if let Some(children) = self.children.get_mut(&parent) {
            children.remove(&child);
        }
    }

    /// Remove all children from a parent node.
    pub fn clear(&mut self, parent: EntityId) {
        if let Some(children) = self.children.remove(&parent) {
            for child in children.iter() {
                self.parents.remove(&child);
            }
        }
    }

    /// Inserts a parent child relationship, removing existing parents of child
    /// if needed.
    pub fn add_child(&mut self, parent: EntityId, child: EntityId) {
        // Remove any existing parents.
        self.graft(child);

        // Insert the child into the parents child set.
        self.children.entry(parent).or_default().insert(child);

        // Map the child id to its parent.
        self.parents.insert(child, parent);
    }
}

impl Scene {
    pub fn hierarchy(&self) -> &Hierarchy {
        &self.hierarchy
    }

    pub fn hierarchy_mut(&mut self) -> &Hierarchy {
        &mut self.hierarchy
    }
}
