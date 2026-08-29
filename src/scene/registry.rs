//! Helper object to store components in cache friendly buckets.

mod query;

pub use query::Query;

use crate::scene::EntityId;
use std::any::{Any, TypeId};
use std::cmp::Ordering;
use std::collections::HashMap;

/// A registry will keep track of attachments to entities.
#[derive(Debug, Default)]
pub struct Registry {
    component_stores: HashMap<TypeId, Vec<(EntityId, Box<dyn Any>)>>,
}

impl Registry {
    /// Attaches `value` to entity, or replaces it if there is already a an
    /// attachment of type `T`.
    pub fn attach<T: Any>(&mut self, id: EntityId, value: T) {
        let type_id = TypeId::of::<T>();
        let store = self.component_stores.entry(type_id).or_default();

        // Keep the vector sorted by entity id.
        let index = store.binary_search_by(|x| x.0.cmp(&id));
        match index {
            Ok(i) => store[i].1 = Box::new(value),
            Err(i) => store.insert(i, (id, Box::new(value))),
        };
    }

    pub fn get<T: Any>(&self, id: EntityId) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        let store = self.component_stores.get(&type_id)?;

        let index = store.binary_search_by(|x| x.0.cmp(&id));
        match index {
            Ok(i) => store.get(i).map(|x| x.1.downcast_ref()).flatten(),
            _ => None,
        }
    }

    pub fn get_mut<T: Any>(&mut self, id: EntityId) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        let store = self.component_stores.get_mut(&type_id)?;

        let index = store.binary_search_by(|x| x.0.cmp(&id));
        match index {
            Ok(i) => store.get_mut(i).map(|x| x.1.downcast_mut()).flatten(),
            _ => None,
        }
    }

    pub fn get_or_default<'a, T: Any + Default>(&'a mut self, id: EntityId) -> &'a mut T {
        let type_id = TypeId::of::<T>();
        let store = self.component_stores.entry(type_id).or_default();

        let index = match store.binary_search_by(|x| x.0.cmp(&id)) {
            Ok(i) => i,
            Err(i) => {
                store.insert(i, (id, Box::new(T::default())));
                i
            }
        };

        store[index]
            .1
            .downcast_mut()
            .expect("bucket for TypeId::of<T>() only ever holds T")
    }

    pub fn has<T: Any>(&self, id: EntityId) -> bool {
        let type_id = TypeId::of::<T>();
        let Some(store) = self.component_stores.get(&type_id) else {
            return false;
        };

        match store.binary_search_by(|x| x.0.cmp(&id)) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn remove<T: Any>(&mut self, id: EntityId) -> Option<T> {
        let type_id = TypeId::of::<T>();
        let Some(store) = self.component_stores.get_mut(&type_id) else {
            return None;
        };

        match store.binary_search_by(|x| x.0.cmp(&id)) {
            Ok(i) => store.remove(i).1.downcast::<T>().ok().map(|x| *x),
            _ => None,
        }
    }

    /// Removes all attached components for this entity, or a noop if there are none.
    pub fn delete(&mut self, entity: EntityId) {
        for (_, bucket) in self.component_stores.iter_mut() {
            if let Ok(i) = bucket.binary_search_by(|pair| pair.0.cmp(&entity)) {
                bucket.remove(i);
            }
        }
    }

    /// Copies all components from `source` onto `target`, replacing as necessary.
    pub fn clone(&mut self, source: EntityId, target: EntityId) {
        for (_, bucket) in self.component_stores.iter_mut() {
            // First, find the index of the existing entity in the store.
            let Ok(index) = bucket.binary_search_by(|pair| pair.0.cmp(&source)) else {
                continue;
            };

            let x = &bucket[index].1;
        }
    }

    /// Returns an iterator over all entities with a given type. Preferable
    pub fn view<T: Any>(&self) -> impl Iterator<Item = (EntityId, &T)> {
        self.component_stores
            .get(&TypeId::of::<T>())
            .into_iter()
            .flatten()
            .filter_map(|pair| pair.1.downcast_ref::<T>().map(|x| (pair.0, x)))
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

        // Output is sorted, so we can keep a pointer.
        let (mut i, mut j) = (0, 0);
        let mut out = Vec::new();
        while i < a.len() && j < b.len() {
            match a[i].0.cmp(&b[j].0) {
                Ordering::Less => i += 1,
                Ordering::Greater => j += 1,
                Ordering::Equal => {
                    let ra = a[i].1.downcast_ref::<A>().expect("A bucket holds only A");
                    let rb = b[j].1.downcast_ref::<B>().expect("B bucket holds only B");
                    out.push((a[i].0, ra, rb));
                    i += 1;
                    j += 1;
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

        // Output is sorted, so we can keep three points and iterate.
        let (mut i, mut j, mut k) = (0, 0, 0);
        let mut out = Vec::new();
        while i < a.len() && j < b.len() && k < c.len() {
            let (ia, ib, ic) = (a[i].0, b[j].0, c[k].0);
            let max = ia.max(ib.max(ic));

            // If any are less than the max, then we need to increment them.
            if ia < max {
                i += 1;
            } else if ib < max {
                j += 1;
            } else if ic < max {
                k += 1;
            } else {
                out.push((
                    ia,
                    a[i].1.downcast_ref().expect("A bucket only holds A"),
                    b[j].1.downcast_ref().expect("B bucket only holds B"),
                    c[k].1.downcast_ref().expect("C bucket only holds C"),
                ));
                i += 1;
                j += 1;
                k += 1;
            }
        }
        out
    }

    pub fn view_tuple<T: Query>(&self) -> T::Item<'_> {
        T::query(self)
    }
}
