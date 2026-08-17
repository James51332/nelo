//! API for grouping entities by their transform
use crate::scene::{EntityId, EntityRef, Label, Scene, Transform, Transformable};
use crate::timeline::TimelineSpline;
use glam::Vec2;

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
    pub fn new(scene: &'a mut Scene, id: EntityId) -> Self {
        Self { scene, id }
    }

    /// Returns the group object
    pub fn group(&self) -> &Group {
        self.scene
            .registry
            .get(self.id)
            .expect("No group attached to this entity")
    }

    pub fn group_mut(&mut self) -> &mut Group {
        self.scene
            .registry
            .get_mut(self.id)
            .expect("No group attached to this entity")
    }

    /// Returns true if this Group contains `id` as a child, and false otherwise.
    pub fn contains(&self, id: EntityId) -> bool {
        self.group().children.contains(&id)
    }

    /// Returns the children of this group and consumes the group.
    pub fn children(&self) -> Vec<EntityId> {
        self.group().children.clone()
    }

    /// Returns the number of children in this group.
    pub fn len(&self) -> usize {
        self.group().children.len()
    }

    /// Adds the given entity into this group under three conditions if the given
    /// entity does not have a parent, and the given entity is not an ancestor of
    /// `self`.
    pub fn add(&mut self, id: EntityId) -> &mut Self {
        // Return if the child has a parent. Ancestry check handled by Transform::parent().
        let mut child = self.scene.registry.get_or_default::<Transform>(id).clone();
        if child.has_parent() {
            return self;
        }

        // Add this to our group.
        let transform = self.scene.registry.get_or_default::<Transform>(self.id);
        if child.parent(Some(transform.clone())).is_ok() {
            self.group_mut().children.push(id);
        }

        self
    }

    /// Removes the given entity from this group, or a no-op if this entity
    /// is not part of the group.
    pub fn remove(&mut self, id: EntityId) -> &mut Self {
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
        self.group_mut().children.retain(|&x| x != id);

        self
    }

    /// Moves all entities in this group to the origin. Useful for resetting
    /// local translations before calling `GroupRef::arrange()`.
    pub fn collapse(&mut self) -> &mut Self {
        self.for_each(|_, mut e| {
            e.transform().to(Vec2::ZERO);
            e
        })
    }

    /// Arranges the elements of this group into a row with given `spacing`.
    pub fn row(&mut self, spacing: f32) -> &mut Self {
        self.space(Vec2::new(spacing, 0.0))
    }

    /// Arranges the elements of this group into a col with given `spacing`.
    pub fn col(&mut self, spacing: f32) -> &mut Self {
        self.space(Vec2::new(0.0, spacing))
    }

    /// Spaces elements equally using given `spacing`, keeping items centered.
    /// Spacing is given in group space.
    pub fn space(&mut self, spacing: Vec2) -> &mut Self {
        let n = self.len();
        if n == 0 {
            return self;
        }

        self.for_each(move |i, e| {
            let offset_index = i as f32 - (n - 1) as f32 / 2.0;
            let offset = spacing * offset_index;
            e.translate(offset)
        })
    }

    /// Spaces entities equally along path parameter alpha from [0, 1].
    pub fn arrange(&mut self, spline: impl Into<TimelineSpline>) -> &mut Self {
        // Make sure we have a child to go along.
        let n = self.len();
        if n == 0 {
            return self;
        }

        // Prepare our timeline, which we move into the loop closure.
        let timeline = spline.into().inner();

        // Transform each along the path.
        self.for_each(move |i, e| {
            let timeline = timeline.clone();
            let a = if n == 1 { 0.5 } else { i as f32 / n as f32 };
            e.translate(move |t| timeline.sample(t).sample(a))
        })
    }

    /// Calls the closure `n` times passing the scene mutably. Useful for
    /// creating multiple entities within the group.
    pub fn create<T>(&mut self, n: u32, mut generate: T) -> &mut Self
    where
        T: for<'b> FnMut(u32, &'b mut Scene) -> EntityRef<'b>,
    {
        let mut group = self;
        for i in 0..n {
            let id = generate(i, group.scene).id();
            group = group.add(id);
        }
        group
    }

    /// Create a single entity which is attached to this group.
    pub fn create_once<T>(&mut self, generate: T) -> &mut Self
    where
        T: for<'b> FnOnce(&'b mut Scene) -> EntityRef<'b>,
    {
        let id = generate(&mut self.scene).id();
        self.add(id)
    }

    /// Runs a function for each element in
    pub fn for_each<F>(&mut self, mut generate: F) -> &mut Self
    where
        F: for<'b> FnMut(u32, EntityRef<'b>) -> EntityRef<'b>,
    {
        for (i, &id) in self.children().iter().enumerate() {
            let entity = EntityRef::new(&mut self.scene, id);
            generate(i as u32, entity);
        }

        self
    }

    // Returns all entities with a matching label.
    pub fn index(&mut self, label: impl Into<Label>) -> Vec<EntityId> {
        let label = label.into();
        let mut children = self.children();

        // Retain children which have a matching label
        children.retain(|&id| {
            self.scene
                .component::<Label>(id)
                .is_some_and(|l| *l == label)
        });

        children
    }

    /// Split this group into subgroups based on a token.
    pub fn split_after(&mut self, label: impl Into<Label>) -> &mut Self {
        let ids = self.index(label);
        self.split_after_ids(&ids)
    }

    /// Splits this group by entity ids. Split ids are not placed into a subgroup.
    pub fn split_after_ids(&mut self, ids: &[EntityId]) -> &mut Self {
        // Remove all the children.
        let children = self.children();
        for &id in children.iter() {
            self.remove(id);
        }

        // Create our initial subgroup.
        let mut subgroup_id = self.scene.group().id();
        self.add(subgroup_id);

        // Iterate over all the children.
        for id in children.into_iter() {
            // Add to the current subgroup.
            GroupRef::new(self.scene, subgroup_id).add(id);

            // Create a new subgroup after any splits.
            if ids.contains(&id) {
                subgroup_id = self.scene.group().id();
                self.add(subgroup_id);
            }
        }

        self
    }

    /// Converts this group into an EntityRef.
    pub fn entity(self) -> EntityRef<'a> {
        EntityRef::new(self.scene, self.id)
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
            Some(GroupRef::new(self.scene, self.id))
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
