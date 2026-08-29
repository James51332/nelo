//! Implement querying for the registry.

use std::any::Any;

use crate::scene::{EntityId, registry::Registry};

// ----- Registry -----

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
