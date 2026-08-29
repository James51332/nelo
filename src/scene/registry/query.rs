//! Implement querying for the registry.

use std::any::Any;

use crate::scene::{EntityId, Scene, registry::Registry};

// ----- Query -----

pub trait Query {
    type Item<'a>;
    fn query<'a>(registry: &'a Registry) -> Self::Item<'a>;
}

impl<A: Any> Query for (A,) {
    // impl Iterator is not stable for trait types, so we return a new vector.
    type Item<'a> = Vec<(EntityId, &'a A)>;

    fn query<'a>(registry: &'a Registry) -> Self::Item<'a> {
        registry.view().collect()
    }
}

impl<A: Any, B: Any> Query for (A, B) {
    type Item<'a> = Vec<(EntityId, &'a A, &'a B)>;

    fn query<'a>(registry: &'a Registry) -> Self::Item<'a> {
        registry.view_pair()
    }
}

impl<A: Any, B: Any, C: Any> Query for (A, B, C) {
    type Item<'a> = Vec<(EntityId, &'a A, &'a B, &'a C)>;

    fn query<'a>(registry: &'a Registry) -> Self::Item<'a> {
        registry.view_triple()
    }
}

impl<A: Any, B: Any, C: Any, D: Any> Query for (A, B, C, D) {
    type Item<'a> = Vec<(EntityId, &'a A, &'a B, &'a C, &'a D)>;

    fn query<'a>(registry: &'a Registry) -> Self::Item<'a> {
        let driver = registry.view::<A>();
        driver
            .filter_map(|pair| {
                let id = pair.0;
                Some((
                    id,
                    pair.1,
                    registry.get::<B>(id)?,
                    registry.get::<C>(id)?,
                    registry.get::<D>(id)?,
                ))
            })
            .collect()
    }
}

impl<A: Any, B: Any, C: Any, D: Any, E: Any> Query for (A, B, C, D, E) {
    type Item<'a> = Vec<(EntityId, &'a A, &'a B, &'a C, &'a D, &'a E)>;

    fn query<'a>(registry: &'a Registry) -> Self::Item<'a> {
        let driver = registry.view::<A>();
        driver
            .filter_map(|pair| {
                let id = pair.0;
                Some((
                    id,
                    pair.1,
                    registry.get::<B>(id)?,
                    registry.get::<C>(id)?,
                    registry.get::<D>(id)?,
                    registry.get::<E>(id)?,
                ))
            })
            .collect()
    }
}

// ----- Query -----

impl Scene {
    /// Returns the attached component of specific type.
    pub fn component<T: Any>(&self, entity: EntityId) -> Option<&T> {
        self.registry.get::<T>(entity)
    }

    /// Returns all attached data of a certain type sorted by EntityId.
    pub fn view<T: Any>(&self) -> impl Iterator<Item = (EntityId, &T)> {
        self.registry.view()
    }

    /// Returns all entities an attached data for entities with components of type
    /// `A` and `B` attached.
    pub fn view_pair<A: Any, B: Any>(&self) -> Vec<(EntityId, &A, &B)> {
        self.registry.view_pair()
    }

    /// Returns all entities an attached data for entities with components of type
    /// `A`, `B`, and `C` attached.
    pub fn view_triple<A: Any, B: Any, C: Any>(&self) -> Vec<(EntityId, &A, &B, &C)> {
        self.registry.view_triple()
    }

    /// Returns a Vector of entities and their attached components which have
    /// up to five types specified by the generic tuple `T`.
    pub fn view_tuple<T: Query>(&self) -> T::Item<'_> {
        self.registry.view_tuple::<T>()
    }
}
