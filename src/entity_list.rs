use std::any::TypeId;
use std::convert::TryInto;


use hashbrown::HashMap;
use hibitset::{BitSet};

use generational_arena::{Arena, Index};

use crate::{EntityBase, Component};

pub type EntityId = Index;

pub struct EntityList<E: EntityBase> {
    pub (crate) bitsets: HashMap<TypeId, BitSet>,
    pub (crate) entities: Arena<E>,
}

impl<E: EntityBase> EntityList<E> {
    pub fn new() -> EntityList<E> {
        let mut l = EntityList {
            bitsets: HashMap::new(),
            entities: Arena::new(),
        };
        l.init_bitsets(None);
        l
    }

    pub fn from_arena(arena: Arena<E>) -> EntityList<E> {
        let mut l: EntityList<_> = EntityList {
            bitsets: HashMap::new(),
            entities: arena,
        };
        l.regenerate_all_component_bitsets();
        l
    }

    pub fn insert(&mut self, entity: E) -> EntityId {
        let mut type_ids: Vec<TypeId> = Vec::with_capacity(8);
        entity.for_each_active_component(|type_id: TypeId| {
            type_ids.push(type_id);
        });
        let entity_id = self.entities.insert(entity);
        let (generation_less_index, _) = entity_id.into_raw_parts();
        for type_id in type_ids {
            if let Some(bitset) = self.bitsets.get_mut(&type_id) {
                bitset.add(generation_less_index as u32);
            }
        }
        entity_id
    }

    pub fn remove(&mut self, id: EntityId) -> Option<E> {
        if let Some(e) = self.entities.remove(id) {
            let generation_less_index = id.into_raw_parts().0;
            e.for_each_active_component(|type_id: TypeId| {
                if let Some(bitset) = self.bitsets.get_mut(&type_id) {
                    bitset.remove(generation_less_index as u32);
                }
            });
            Some(e)
        } else {
            None
        }
    }

    #[inline]
    pub fn get(&self, id: EntityId) -> Option<&E> {
        self.entities.get(id)
    }

    #[inline]
    /// Retrieves an entity mutably.
    ///
    /// **WARNING**: You must not add or remove a component to this entity via the mutable
    /// reference, otherwise the bitset cache will be invalid, resulting in this entity
    /// possibly not being iterated over!
    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut E> {
        self.entities.get_mut(id)
    }

    #[inline]
    pub fn contains(&self, id: EntityId) -> bool {
        self.entities.contains(id)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Initialize bitsets for all components of entity E
    ///
    /// Capacity is 4096, and is applied for all bitsets.
    pub (crate) fn init_bitsets(&mut self, capacity: Option<u32>) {
        E::for_all_components(|type_id: TypeId| {
            self.bitsets.insert(type_id, BitSet::with_capacity(capacity.unwrap_or(4096)));
        });
    }

    fn regenerate_all_component_bitsets(&mut self) {
        let capacity = self.entities.len();
        E::for_all_components(|type_id: TypeId| {
            self.bitsets.insert(type_id, BitSet::with_capacity(capacity as u32));
        });
        for (id, el) in &self.entities {
            let i = id.into_raw_parts().0;
            let bitsets: &mut HashMap<TypeId, BitSet> = &mut self.bitsets;
            el.for_each_active_component(|type_id: TypeId| {
                let bitset: &mut BitSet = bitsets.get_mut(&type_id).unwrap();
                bitset.add(i as u32);
            })
        }
    }

    // Add a bitset for a specific component for all entities.
    //
    // Typically done at the very start of the ECS
    #[allow(dead_code)]
    pub (crate) fn add_bitset_for_component<C: Component<E>>(&mut self) {
        let bitset_capacity: u32 = self.entities.capacity().try_into().expect("too many entities");
        let mut bitset = BitSet::with_capacity(bitset_capacity);
        for (entity_id, entity) in &self.entities {
            if entity.has::<C>() {
                bitset.add(entity_id.into_raw_parts().0 as u32);
            }
        }
        self.bitsets.insert(
            TypeId::of::<C>(),
            bitset
        );
    }

    // Remove a bitset for a specific component for all entities.
    //
    // Returns true if the bitset was actually there and was removed
    #[allow(dead_code)]
    pub (crate) fn remove_bitset_for_component<C: Component<E>>(&mut self) -> bool {
        let bitset_capacity: u32 = self.entities.capacity().try_into().expect("too many entities");
        let mut bitset = BitSet::with_capacity(bitset_capacity);
        for (entity_id, entity) in &self.entities {
            if entity.has::<C>() {
                bitset.remove(entity_id.into_raw_parts().0 as u32);
            }
        }
        self.bitsets.remove(
            &TypeId::of::<C>()
        ).is_some()
    }

    /// Add a component for the given entity.
    ///
    /// If the entity does not exist anymore, Some(component) is returned.
    pub fn add_component_for_entity<C: Component<E> + 'static>(&mut self, entity_id: EntityId, component: C) -> Option<C> {
        let maybe_component = match self.entities.get_mut(entity_id) {
            Some(e) => {
                component.set(e);
                None
            },
            None => {
                Some(component)
            }
        };
        // maybe_component is Some if it hasn't been applied, None if it has been applied.
        if maybe_component.is_none() {
            // if it has been added, see if we have a bitset for this component
            if let Some(bitset) = self.bitsets.get_mut(&TypeId::of::<C>()) {
                // we have a bitset, so add the info that this entity has the given component
                bitset.add(entity_id.into_raw_parts().0 as u32);
            };
        };

        maybe_component
    }

    /// Remove a component for the given entity.
    ///
    /// If the entity exists and it has the component, Some(component) is returned.
    pub fn remove_component_for_entity<C: Component<E> + 'static>(&mut self, entity_id: EntityId) -> Option<Box<C>> {
        let maybe_component = self.entities
            .get_mut(entity_id)
            .and_then(C::remove);

        // maybe_component is Some if it was a component, None if it wasn't.
        if maybe_component.is_some() {
            // if it has been removed, see if we have a bitset for this component
            if let Some(bitset) = self.bitsets.get_mut(&TypeId::of::<C>()) {
                // we have a bitset, so remove the info that this entity has the given component
                bitset.remove(entity_id.into_raw_parts().0 as u32);
            };
        };

        maybe_component
    }

    /// Akin to Vec::retain, deletes entities where the predicate returns true
    pub fn retain(&mut self, mut predicate: impl FnMut(EntityId, &mut E) -> bool) {
        let bitsets = &mut self.bitsets;
        self.entities.retain(|index, e| {
            let should_delete = predicate(index, e);
            if should_delete {
                e.for_each_active_component(|type_id: TypeId| {
                    if let Some(bitset) = bitsets.get_mut(&type_id) {
                        bitset.remove(index.clone().into_raw_parts().0 as u32);
                    }
                });
            }
            should_delete
        })
    }
}