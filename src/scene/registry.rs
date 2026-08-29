//! Helper object to store components in cache friendly buckets.

mod query;

pub use query::Query;

use crate::scene::EntityId;
use std::any::{Any, TypeId};
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};

type Store = BTreeMap<EntityId, Box<dyn Any>>;

/// A registry will keep track of attachments to entities.
#[derive(Debug, Default)]
pub struct Registry {
    component_stores: HashMap<TypeId, Store>,
}

impl Registry {
    /// Attaches `value` to entity, or replaces it if there is already a an
    /// attachment of type `T`.
    pub fn attach<T: Any>(&mut self, id: EntityId, value: T) {
        let type_id = TypeId::of::<T>();
        let store = self.component_stores.entry(type_id).or_default();

        // Keep the vector sorted by entity id.
        store.insert(id, Box::new(value));
    }

    pub fn get<T: Any>(&self, id: EntityId) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        let store = self.component_stores.get(&type_id)?;
        store.get(&id).map(|x| x.downcast_ref()).flatten()
    }

    pub fn get_mut<T: Any>(&mut self, id: EntityId) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        let store = self.component_stores.get_mut(&type_id)?;
        store.get_mut(&id).map(|x| x.downcast_mut()).flatten()
    }

    pub fn get_or_default<'a, T: Any + Default>(&'a mut self, id: EntityId) -> &'a mut T {
        let type_id = TypeId::of::<T>();
        let store = self.component_stores.entry(type_id).or_default();
        store
            .entry(id)
            .or_insert(Box::new(T::default()))
            .downcast_mut()
            .expect("bucket for TypeId::of<T>() only ever holds T")
    }

    pub fn has<T: Any>(&self, id: EntityId) -> bool {
        let type_id = TypeId::of::<T>();
        if let Some(store) = self.component_stores.get(&type_id) {
            store.contains_key(&id)
        } else {
            false
        }
    }

    pub fn remove<T: Any>(&mut self, id: EntityId) -> Option<T> {
        let type_id = TypeId::of::<T>();
        if let Some(store) = self.component_stores.get_mut(&type_id) {
            store
                .remove(&id)
                .map(|x| x.downcast::<T>().ok())
                .flatten()
                .map(|x| *x)
        } else {
            None
        }
    }

    /// Removes all attached components for this entity, or a noop if there are none.
    pub fn delete(&mut self, entity: EntityId) {
        for (_, bucket) in self.component_stores.iter_mut() {
            bucket.remove(&entity);
        }
    }

    /// Returns an iterator over all entities with a given type.
    pub fn view<T: Any>(&self) -> impl Iterator<Item = (EntityId, &T)> {
        self.component_stores
            .get(&TypeId::of::<T>())
            .into_iter()
            .flatten()
            .filter_map(|pair| pair.1.downcast_ref::<T>().map(|x| (*pair.0, x)))
    }

    /// Returns a vector over all entities with at least two types.
    pub fn view_pair<A: Any, B: Any>(&self) -> Vec<(EntityId, &A, &B)> {
        let (a, b) = match (
            self.component_stores.get(&TypeId::of::<A>()),
            self.component_stores.get(&TypeId::of::<B>()),
        ) {
            (Some(a), Some(b)) => (a, b),
            _ => return Vec::new(),
        };

        let mut iter_a = a.iter();
        let mut iter_b = b.iter();
        let mut next_a = iter_a.next();
        let mut next_b = iter_b.next();
        let mut out = Vec::new();

        while let (Some((id_a, a)), Some((id_b, b))) = (next_a, next_b) {
            match id_a.cmp(id_b) {
                Ordering::Less => next_a = iter_a.next(),
                Ordering::Greater => next_b = iter_b.next(),
                Ordering::Equal => {
                    let ra = a.downcast_ref::<A>().expect("A bucket holds only A");
                    let rb = b.downcast_ref::<B>().expect("B bucket holds only B");
                    out.push((*id_a, ra, rb));
                    next_a = iter_a.next();
                    next_b = iter_b.next();
                }
            }
        }
        out
    }

    pub fn view_triple<A: Any, B: Any, C: Any>(&self) -> Vec<(EntityId, &A, &B, &C)> {
        let (a, b, c) = match (
            self.component_stores.get(&TypeId::of::<A>()),
            self.component_stores.get(&TypeId::of::<B>()),
            self.component_stores.get(&TypeId::of::<C>()),
        ) {
            (Some(a), Some(b), Some(c)) => (a, b, c),
            _ => return Vec::new(),
        };

        let mut iter_a = a.iter();
        let mut iter_b = b.iter();
        let mut iter_c = c.iter();
        let mut next_a = iter_a.next();
        let mut next_b = iter_b.next();
        let mut next_c = iter_c.next();
        let mut out = Vec::new();

        while let (Some((id_a, a)), Some((id_b, b)), Some((id_c, c))) = (next_a, next_b, next_c) {
            let max = id_a.max(id_b.max(id_c));

            if id_a < max {
                next_a = iter_a.next();
            } else if id_b < max {
                next_b = iter_b.next();
            } else if id_c < max {
                next_c = iter_c.next();
            } else {
                let a = a.downcast_ref::<A>().expect("A bucket holds only A");
                let b = b.downcast_ref::<B>().expect("B bucket holds only B");
                let c = c.downcast_ref::<C>().expect("C bucket holds only C");
                out.push((*id_a, a, b, c));
                next_a = iter_a.next();
                next_b = iter_b.next();
                next_c = iter_c.next();
            }
        }
        out
    }

    pub fn view_tuple<T: Query>(&self) -> T::Item<'_> {
        T::query(self)
    }
}
