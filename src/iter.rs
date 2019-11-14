use crate::{Component, EntityBase, EntityList, EntityId};
use generational_arena::Arena;
use hibitset::{BitIter, BitSet, BitSetLike, BitSetAll, BitSetAnd};

use std::any::TypeId;

use hashbrown::HashMap;


impl<E: EntityBase> EntityList<E> {
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