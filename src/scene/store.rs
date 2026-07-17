//! Helper object to store entity data and components.

use crate::render::Geometry;
use crate::scene::{EntityData, EntityId, EntityRef};
use std::collections::HashMap;

pub(crate) struct Store {
    entity_data: HashMap<EntityId, EntityData>,
    next_key: usize,
}

impl Store {
    pub fn new() -> Self {
        Self {
            entity_data: HashMap::new(),
            next_key: 0,
        }
    }

    pub fn create(&mut self, geometry: Geometry) -> EntityRef<'_> {
        let id = EntityId::new(self.next_key);
        self.next_key = self.next_key + 1;
        EntityRef::new(
            self.entity_data
                .entry(id)
                .or_insert(EntityData::new(geometry)),
            id,
        )
    }

    pub fn delete(&mut self, id: EntityId) {
        self.entity_data.remove(&id);
    }

    pub fn get(&mut self, id: EntityId) -> Option<EntityRef<'_>> {
        self.entity_data.get_mut(&id).map(|x| EntityRef::new(x, id))
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, EntityId, EntityData> {
        self.entity_data.iter()
    }
}
