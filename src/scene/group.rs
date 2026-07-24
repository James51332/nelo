//! API for grouping entities by their transform
use crate::scene::{EntityId, EntityRef, Scene, Transform, Transformable};

/// A group is a transform that can be applied to children. Adding it
/// to an entity will not affect rendering for that entity, but only for
/// the children.
#[derive(Default)]
pub struct Group {
    children: Vec<EntityId>,
}

pub struct GroupRef<'a> {
    scene: &'a mut Scene,
    id: EntityId,
}

impl<'a> GroupRef<'a> {
    /// Returns true if this Group contains `id` as a child, and false otherwise.
    pub fn contains(&self, id: EntityId) -> bool {
        let g: &Group = self.scene.registry.get(self.id).unwrap();
        g.children.contains(&id)
    }

    /// Adds the given entity into this group under three conditions if the given
    /// entity does not have a parent, and the given entity is not an ancestor of
    /// `self`.
    pub fn add(self, id: EntityId) -> Self {
        // Return if the child has a parent. Ancestry check handled by Transform::parent().
        let mut child = self.scene.registry.get_or_default::<Transform>(id).clone();
        if child.has_parent() {
            return self;
        }

        // Add this to our group.
        let transform = self.scene.registry.get_or_default::<Transform>(self.id);
        if child.parent(Some(transform.clone())).is_ok() {
            let group: &mut Group = self.scene.registry.get_mut(self.id).unwrap();
            group.children.push(id);
        }

        self
    }

    /// Removes the given entity from this group, or a no-op if this entity
    /// is not part of the group.
    pub fn remove(self, id: EntityId) -> Self {
        // No-op if we don't contain the entity.
        if !self.contains(id) {
            return self;
        }

        // Remove the parent.
        let _ = self
            .scene
            .registry
            .get_or_default::<Transform>(id)
            .parent(None);

        // Update our children.
        self.scene
            .registry
            .get_mut::<Group>(self.id)
            .unwrap()
            .children
            .retain(|&x| x != id);

        self
    }

    /// Calls the closure `n` times passing the scene mutably. Useful for
    /// creating multiple entities within the group.
    pub fn create<T>(self, n: u32, mut generate: T) -> Self
    where
        T: FnMut(u32, &mut Scene) -> EntityId,
    {
        let mut group = self;
        for i in 0..n {
            let id = generate(i, group.scene);
            group = group.add(id);
        }
        group
    }

    /// Create a single entity which is attached to this group.
    pub fn create_once<T>(mut self, generate: T) -> Self
    where
        T: FnOnce(&mut Scene) -> EntityId,
    {
        let id = generate(&mut self.scene);
        self.add(id)
    }

    /// Drops this reference and returns the entity id of this group.
    pub fn id(self) -> EntityId {
        self.id
    }
}

impl Transformable for GroupRef<'_> {
    fn transform(&mut self) -> &mut Transform {
        self.scene.registry.get_or_default(self.id)
    }
}

// ----- EntityRef -----

impl<'a> EntityRef<'a> {
    /// Return this entity as a group if it is one, or None if it not a group.
    pub fn as_group(self) -> Option<GroupRef<'a>> {
        if self.has::<Group>() {
            Some(GroupRef {
                scene: self.scene,
                id: self.id,
            })
        } else {
            None
        }
    }
}

// ----- Scene -----

impl Scene {
    /// Creates a group.
    pub fn group(&mut self) -> GroupRef<'_> {
        self.create().attach(Group::default()).as_group().unwrap()
    }
}
