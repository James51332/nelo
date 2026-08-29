//! API for grouping entities by their transform
use crate::scene::{EntityId, EntityRef, Hierarchy, Label, Scene, Transform, Transformable};
use crate::timeline::TimelineSpline;
use glam::{Affine2, Vec2};

pub struct GroupRef<'a> {
    scene: &'a mut Scene,
    id: EntityId,
}

impl<'a> GroupRef<'a> {
    pub(super) fn new(scene: &'a mut Scene, id: EntityId) -> Self {
        Self { scene, id }
    }

    /// Returns the hierarchy for the scene.
    pub fn hierarchy(&self) -> &Hierarchy {
        &self.scene.hierarchy
    }

    /// Returns the children of this group.
    pub fn children(&self) -> Vec<EntityId> {
        self.hierarchy().children(self.id)
    }

    /// Returns the number of children in this group.
    pub fn len(&self) -> usize {
        self.children().len()
    }

    /// Returns true if this Group contains `id` as a child, and false otherwise.
    pub fn contains(&self, id: EntityId) -> bool {
        self.children().contains(&id)
    }

    /// Adds `entity` to this group, grafting it from its parent if needed.
    pub fn add(&mut self, child: EntityId) -> &mut Self {
        let hierarchy = &mut self.scene.hierarchy;
        hierarchy.graft(child);
        hierarchy.add_child(self.id, child);
        self
    }

    /// Removes `entity` from this group, or no-op if it is not part of the group.
    pub fn remove(&mut self, child: EntityId) -> &mut Self {
        let hierarchy = &mut self.scene.hierarchy;
        if hierarchy.is_parent(self.id, child) {
            hierarchy.graft(child);
        }

        self
    }

    /// Grafts all children in this group.
    pub fn ungroup(&mut self) -> Vec<EntityId> {
        let hierarchy = &mut self.scene.hierarchy;
        let children = hierarchy.children(self.id);

        for &child in children.iter() {
            hierarchy.graft(child);
        }

        children
    }

    /// Moves all entities in this group to the origin. Useful for resetting
    /// local translations before calling `GroupRef::arrange()`.
    pub fn collapse(&mut self) -> &mut Self {
        self.for_each(|_, mut e| {
            e.transform().reset();
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
        for i in 0..n {
            let id = generate(i, self.scene).id();
            self.add(id);
        }

        self
    }

    /// Create a single entity which is attached to this group.
    pub fn create_once<T>(&mut self, generate: T) -> &mut Self
    where
        T: for<'b> FnOnce(&'b mut Scene) -> EntityRef<'b>,
    {
        let id = generate(&mut self.scene).id();
        self.add(id)
    }

    /// Runs a function for each element in the group.
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

    /// Returns the element at given index, or panics if out of bounds.
    pub fn index(&mut self, index: usize) -> EntityId {
        self.children()[index]
    }

    /// Returns all entities with a matching label.
    pub fn labeled(&mut self, label: impl Into<Label>) -> Vec<EntityId> {
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

    pub fn split_before(&mut self, label: impl Into<Label>) -> &mut Self {
        let ids = self.labeled(label);
        self.split_before_ids(&ids)
    }

    /// Splits this group at all entities matching the label.
    pub fn split(&mut self, label: impl Into<Label>) -> &mut Self {
        let ids = self.labeled(label);
        self.split_at_ids(&ids)
    }

    /// Split this group into subgroups based on a token.
    pub fn split_after(&mut self, label: impl Into<Label>) -> &mut Self {
        let ids = self.labeled(label);
        self.split_after_ids(&ids)
    }

    pub fn split_before_ids(&mut self, ids: &[EntityId]) -> &mut Self {
        let children = self.ungroup();
        let mut subgroup_id = self.scene.group().id();
        self.add(subgroup_id);

        for id in children.into_iter() {
            if ids.contains(&id) {
                subgroup_id = self.scene.group().id();
                self.add(subgroup_id);
            }

            GroupRef::new(self.scene, subgroup_id).add(id);
        }

        self
    }

    /// Divide into subgroups. entities in given slice remain at top-level.
    pub fn split_at_ids(&mut self, ids: &[EntityId]) -> &mut Self {
        let children = self.ungroup();

        let mut subgroup_id = self.scene.group().id();
        self.add(subgroup_id);

        for id in children.into_iter() {
            if ids.contains(&id) {
                // Add split entities back into this group.
                self.add(id);

                // And create a new subgroup.
                subgroup_id = self.scene.group().id();
                self.add(subgroup_id);
            } else {
                // Add non-split entities to the subgroup.
                GroupRef::new(self.scene, subgroup_id).add(id);
            }
        }

        self
    }

    /// Splits this group by entity ids. Split elements are placed at the end of subgroups.
    pub fn split_after_ids(&mut self, ids: &[EntityId]) -> &mut Self {
        // Remove all the children.
        let children = self.ungroup();

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

    /// Returns the entity id of this group.
    pub fn id(&self) -> EntityId {
        self.id
    }
}

impl Transformable for &mut GroupRef<'_> {
    fn transform(&mut self) -> &mut Transform {
        self.scene.registry.get_or_default(self.id)
    }
}

// ----- EntityRef -----

impl<'a> EntityRef<'a> {
    /// Access this entity using group API. Does not make any changes to the
    /// entity.
    pub fn as_group(self) -> GroupRef<'a> {
        GroupRef::new(self.scene, self.id)
    }
}

// ----- Scene -----

impl Scene {
    /// Creates a group.
    pub fn group(&mut self) -> GroupRef<'_> {
        self.create().as_group()
    }

    /// Returns the world transform for an entity at a given time, composing with
    /// parent transforms as needed.
    pub fn world_transform(&self, entity: EntityId, time: f32) -> Affine2 {
        // Get the transform for this entity.
        let mut transform = self
            .registry
            .get::<Transform>(entity)
            .map_or(Affine2::default(), |t| t.local(time));

        // Walk up the parents and apply their transforms.
        let mut current = entity;
        while let Some(parent) = self.hierarchy.parent(current) {
            if let Some(parent_transform) = self.registry.get::<Transform>(parent) {
                transform = parent_transform.local(time) * transform;
            }

            current = parent;
        }

        transform
    }
}
