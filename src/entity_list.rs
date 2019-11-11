use std::any::TypeId;
use std::convert::TryInto;


use hashbrown::HashMap;
use hibitset::{BitSet, BitSetLike, BitSetAnd};

use generational_arena::{Arena, Index};

use crate::{EntityBase, Component};

pub type EntityId = Index;

pub struct EntityList<E: EntityBase> {
    bitsets: HashMap<TypeId, BitSet>,
    entities: Arena<E>,
}

impl<E: EntityBase> EntityList<E> {
    pub fn new() -> EntityList<E> {
        EntityList {
            bitsets: HashMap::new(),
            entities: Arena::new(),
        }
    }

    // Add a bitset for a specific component for all entities.
    //
    // Typically done at the very start of the ECS
    pub fn add_bitset_for_component<C: Component<E> + 'static>(&mut self) {
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
    pub fn remove_bitset_for_component<C: Component<E> + 'static>(&mut self) -> bool {
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

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=(EntityId, &'a E)> {
        self.entities.iter()
    }

    pub fn iter_for_components<'a, C: MultiComponent<E>>(&'a self) -> Box<dyn Iterator<Item=(EntityId, &'a E)> + 'a> {
        let bitset = C::join(&self.bitsets);
        let iter = bitset.iter().filter_map(move |i: u32| {
            let maybe = self.entities.get_unknown_gen(i as usize);
            maybe.and_then(|(e, i)| {
                if C::entity_has_components(e) {
                    Some((i, e))
                } else {
                    None
                }
            })
        });
        Box::new(iter)
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

    pub fn add_entity(&mut self, entity: E) -> EntityId {
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

    pub fn remove_entity(&mut self, id: EntityId) -> Option<E> {
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

    pub fn retain(&mut self, mut predicate: impl FnMut(EntityId, &mut E) -> bool) {
        let bitsets = &mut self.bitsets;
        self.entities.retain(|index, e| {
            e.for_each_active_component(|type_id: TypeId| {
                if let Some(bitset) = bitsets.get_mut(&type_id) {
                    bitset.remove(index.clone().into_raw_parts().0 as u32);
                }
            });
            predicate(index, e)
        })
    }
}

pub trait MultiComponent<E: EntityBase> {
    type JoinedBitSet: BitSetLike + 'static;

    fn join(bitsets: &HashMap<TypeId, BitSet>) -> Self::JoinedBitSet;

    fn entity_has_components(entity: &E) -> bool;
}

// impl<E: EntityBase, C: Component<E> + 'static> MultiComponent<E> for C {
//     type JoinedBitSet = BitSet;

//     fn join(bitsets: &HashMap<TypeId, BitSet>) -> Self::JoinedBitSet {
//         match bitsets.get(&TypeId::of::<C>()) {
//             Some(other_bitset) => other_bitset.clone(),
//             None => BitSet::new(),
//         }
//     }

//     fn entity_has_components(entity: &E) -> bool {
//         entity.has::<C>()
//     }
// }

macro_rules! p {
    ($x:ident) => (
        BitSetAnd<BitSet, BitSet>
    );
    ($x:ident, $( $y:ident ),+) => (
        BitSetAnd<p!( $($y), *), BitSet>
    )
}

macro_rules! multi_component_impl {
    ( $($ty:ident),* ) => {
        impl<E: EntityBase, $( $ty : Component<E> + 'static ),*> MultiComponent<E> for ( $( $ty , )* ) {
            type JoinedBitSet = p!($($ty),*);

            fn join(bitsets: &HashMap<TypeId, BitSet>) -> Self::JoinedBitSet {
                let bitset = BitSet::new();
                $(
                let bitset = match bitsets.get(&TypeId::of::<$ty>()) {
                    Some(other_bitset) => BitSetAnd(bitset, other_bitset.clone()),
                    None => BitSetAnd(bitset, BitSet::new()),
                };
                )*
                bitset
            }

            fn entity_has_components(entity: &E) -> bool {
                $( entity.has::<$ty>() && )* true
            }
        }
    }
}

multi_component_impl!(C1);
// multi_component_impl!(C1, C2);
// multi_component_impl!(C1, C2, C3);
// multi_component_impl!(C1, C2, C3, C4);
// multi_component_impl!(C1, C2, C3, C4, C5);
// multi_component_impl!(C1, C2, C3, C4, C5, C6);
// multi_component_impl!(C1, C2, C3, C4, C5, C6, C7);
// multi_component_impl!(C1, C2, C3, C4, C5, C6, C7, C8);
// multi_component_impl!(C1, C2, C3, C4, C5, C6, C7, C8, C9);
// multi_component_impl!(C1, C2, C3, C4, C5, C6, C7, C8, C9, C10);
// multi_component_impl!(C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11);
// multi_component_impl!(C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12);
// multi_component_impl!(C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13);
// multi_component_impl!(C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14);
// multi_component_impl!(C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15);
// multi_component_impl!(C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, C16);
// multi_component_impl!(C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, C16, C17);
// multi_component_impl!(C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, C16, C17, C18);