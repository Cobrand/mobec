use std::any::TypeId;
use std::convert::TryInto;


use hashbrown::HashMap;
use hibitset::{BitSet, BitSetLike, BitIter, BitSetAll, BitSetAnd};

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

    pub fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item=(EntityId, &'a mut E)> {
        self.entities.iter_mut()
    }

    pub fn iter_for_components<'a, C: MultiComponent<E>>(&'a self) -> impl Iterator<Item=(EntityId, &'a E)> {
        let bitset_iter = C::iter(&self.bitsets);
        MultiComponentIter::<E, C>::new(bitset_iter, &self.entities)
    }

    pub fn iter_mut_for_components<'a, C: MultiComponent<E>>(&'a mut self) -> impl Iterator<Item=(EntityId, &'a mut E)> {
        let bitset_iter = C::iter(&self.bitsets);
        MultiComponentIterMut::<E, C>::new(bitset_iter, &mut self.entities)
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

pub struct MultiComponentIter<'a, E: EntityBase, C: MultiComponent<E>> {
    pub (crate) bitset_iter: BitIter<Box<dyn BitSetLike + 'a>>,
    pub (crate) entities: &'a Arena<E>,
    d: std::marker::PhantomData<C>,
}

impl<'a, E: EntityBase, C: MultiComponent<E>> MultiComponentIter<'a, E, C> {
    pub fn new(bitset_iter: BitIter<Box<dyn BitSetLike + 'a>>, entities: &'a Arena<E>) -> MultiComponentIter<'a, E, C> {
        MultiComponentIter {
            bitset_iter,
            entities,
            d: Default::default(),
        }
    }
}

impl<'a, E: EntityBase, C: MultiComponent<E>> Iterator for MultiComponentIter<'a, E, C> {
    type Item = (EntityId, &'a E);
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(n) = self.bitset_iter.next() {
            match self.entities.get_unknown_gen(n as usize) {
                Some((el, index)) if C::entity_has_components(el) => return Some((index, el)),
                _ => continue,
            }
        }
        None
    }
}

pub struct MultiComponentIterMut<'a, E: EntityBase, C: MultiComponent<E>> {
    pub (crate) bitset_iter: BitIter<Box<dyn BitSetLike + 'a>>,
    pub (crate) entities: &'a mut Arena<E>,
    d: std::marker::PhantomData<C>,
    #[cfg(debug_assertions)]
    pub (crate) n: Option<u32>,
}

impl<'a, E: EntityBase, C: MultiComponent<E>> MultiComponentIterMut<'a, E, C> {
    pub fn new(bitset_iter: BitIter<Box<dyn BitSetLike + 'a>>, entities: &'a mut Arena<E>) -> MultiComponentIterMut<'a, E, C> {
        MultiComponentIterMut {
            bitset_iter,
            entities,
            d: Default::default(),
            #[cfg(debug_assertions)]
            n: None,
        }
    }
}

impl<'a, E: EntityBase, C: MultiComponent<E>> Iterator for MultiComponentIterMut<'a, E, C> {
    type Item = (EntityId, &'a mut E);
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(n) = self.bitset_iter.next() {
            // check that n is strictly monotonically increasing, and thus never
            // the same value twice
            #[cfg(debug_assertions)] {
                if let Some(old_n) = self.n {
                    debug_assert!(n > old_n);
                }
                self.n = Some(n);
            }

            match self.entities.get_unknown_gen_mut(n as usize) {
                // We have the guarantee from
                Some((el, index)) if C::entity_has_components(el) => {
                    #[allow(unsafe_code)]
                    // technically, we shouldn't be able to do this since
                    // "next" borrows mut self, and we return a mutable item from self.
                    // However, we have the guarantee that itset_iter.next() never returns
                    // the same number twice, so it IS safe to return multiple references
                    // to the same data in the case of this iterator.
                    let el = unsafe { &mut *(el as *mut _) };
                    return Some((index, el))
                },
                _ => continue,
            }
        }
        None
    }
}

pub trait MultiComponent<E: EntityBase> {
    fn iter<'a>(bitsets: &'a HashMap<TypeId, BitSet>) -> BitIter<Box<dyn BitSetLike + 'a>>;

    fn entity_has_components(entity: &E) -> bool;
}

macro_rules! multi_component_impl {
    ( $($ty:ident),* ) => {
        impl<E: EntityBase, $( $ty : Component<E> + 'static ),*> MultiComponent<E> for ( $( $ty , )* ) {
            fn iter<'a>(bitsets: &'a HashMap<TypeId, BitSet>) -> BitIter<Box<dyn BitSetLike + 'a>> {
                let bitset = BitSetAll;
                $(
                let bitset: Box<dyn BitSetLike> = match bitsets.get(&TypeId::of::<$ty>()) {
                    Some(other_bitset) => Box::new(BitSetAnd(bitset, other_bitset)),
                    None => Box::new(BitSetAnd(bitset, BitSetAll)),
                };
                )*
                BitIter::from(bitset)
            }

            fn entity_has_components(entity: &E) -> bool {
                $( entity.has::<$ty>() && )* true
            }
        }
    }
}

multi_component_impl!(C1);
multi_component_impl!(C1, C2);
multi_component_impl!(C1, C2, C3);
multi_component_impl!(C1, C2, C3, C4);
multi_component_impl!(C1, C2, C3, C4, C5);
multi_component_impl!(C1, C2, C3, C4, C5, C6);
multi_component_impl!(C1, C2, C3, C4, C5, C6, C7);
multi_component_impl!(C1, C2, C3, C4, C5, C6, C7, C8);
multi_component_impl!(C1, C2, C3, C4, C5, C6, C7, C8, C9);
multi_component_impl!(C1, C2, C3, C4, C5, C6, C7, C8, C9, C10);
multi_component_impl!(C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11);
multi_component_impl!(C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12);
multi_component_impl!(C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13);
multi_component_impl!(C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14);
multi_component_impl!(C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15);
multi_component_impl!(C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, C16);