//! API for grouping entities by their transform
use crate::scene::{EntityId, EntityRef, Scene, Transform, Transformable};
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
    /// Returns true if this Group contains `id` as a child, and false otherwise.
    pub fn contains(&self, id: EntityId) -> bool {
        let g: &Group = self.scene.registry.get(self.id).unwrap();
        g.children.contains(&id)
    }

    /// Returns the number of children in this group.
    pub fn len(&self) -> usize {
        let g: &Group = self.scene.registry.get(self.id).unwrap();
        g.children.len()
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

    /// Moves all entities in this group to the origin. Useful for resetting
    /// local translations before calling `GroupRef::arrange()`.
    pub fn collapse(self) -> Self {
        self.for_each(|_, mut e| {
            e.transform().to(Vec2::ZERO);
            e
        })
    }

    /// Arranges the elements of this group into a row with given `spacing`.
    pub fn row(self, spacing: f32) -> Self {
        self.space(Vec2::new(spacing, 0.0))
    }

    /// Arranges the elements of this group into a col with given `spacing`.
    pub fn col(self, spacing: f32) -> Self {
        self.space(Vec2::new(0.0, spacing))
    }

    /// Spaces elements equally using given `spacing`, keeping items centered.
    /// Spacing is given in group space.
    pub fn space(self, spacing: Vec2) -> Self {
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
    pub fn arrange(self, spline: impl Into<TimelineSpline>) -> Self {
        // Make sure we have a child to go along.
        let n = self.len();
        if n == 0 {
            return self;
        }

        // Prepare our timeline, which we move into the loop closure.
        let timeline = spline.into().0.0;

        // Transform each along the path.
        self.for_each(move |i, e| {
            let timeline = timeline.clone();
            let a = if n == 1 { 0.5 } else { i as f32 / n as f32 };
            e.translate(move |t| timeline.sample(t).sample(a))
        })
    }

    /// Calls the closure `n` times passing the scene mutably. Useful for
    /// creating multiple entities within the group.
    pub fn create<T>(self, n: u32, mut generate: T) -> Self
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
    pub fn create_once<T>(mut self, generate: T) -> Self
    where
        T: for<'b> FnOnce(&'b mut Scene) -> EntityRef<'b>,
    {
        let id = generate(&mut self.scene).id();
        self.add(id)
    }

    /// Runs a function for each element in
    pub fn for_each<F>(mut self, mut generate: F) -> Self
    where
        F: for<'b> FnMut(u32, EntityRef<'b>) -> EntityRef<'b>,
    {
        let children = self
            .scene
            .registry
            .get::<Group>(self.id)
            .unwrap()
            .children
            .clone();

        for (i, &id) in children.iter().enumerate() {
            let entity = EntityRef::new(&mut self.scene, id);
            generate(i as u32, entity);
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
