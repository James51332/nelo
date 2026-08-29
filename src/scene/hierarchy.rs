//! A DAG representing the hierarchy within a scene.

use crate::scene::{EntityId, Scene};
use std::collections::HashMap;

#[derive(Default)]
pub struct Hierarchy {
    children: HashMap<EntityId, Vec<EntityId>>,
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
        // Check if these are same entity.
        let mut current = child;
        if current == entity {
            return true;
        }

        // Walk up child's ancestry to find entity.
        while let Some(&parent) = self.parents.get(&current) {
            if parent == entity {
                return true;
            }

            current = parent;
        }

        false
    }

    /// Get all children for a given entity.
    pub fn children(&self, parent: EntityId) -> Vec<EntityId> {
        self.children.get(&parent).cloned().unwrap_or_default()
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
            children.retain(|&id| id != child);
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

    // Grafts the entity and its children.
    pub fn remove(&mut self, entity: EntityId) {
        self.graft(entity);
        self.clear(entity);
    }

    /// Inserts a parent child relationship, removing existing parents of child
    /// if needed. No-op if the child is an ancestor of the parent.
    pub fn add_child(&mut self, parent: EntityId, child: EntityId) {
        // If the child is an ancestor of parent, then we would create a circle
        // dependency. Disallow this.
        if self.is_ancestor(child, parent) {
            return;
        }

        // Remove any existing parents.
        self.graft(child);

        // Insert the child into the parents child set.
        self.children.entry(parent).or_default().push(child);

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
