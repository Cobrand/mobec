use std::any::TypeId;

pub trait Component {
    type Entity: Sized + 'static;

    fn set(self, entity: &mut Self::Entity);

    fn get(entity: &Self::Entity) -> Option<&Self>;

    fn get_mut(entity: &mut Self::Entity) -> Option<&mut Self>;

    /// Delete a component from an entity
    fn remove(entity: &mut Self::Entity) -> Option<Box<Self>>;
    
    // read a component with the given predicate. You may return a custom result of your choice.
    fn peek<O, F: FnOnce(&Self) -> O>(entity: &Entity, f: F) -> Option<O>;

    // update component with the given predicate. You may return a custom result of your choice.
    fn update<O, F: FnOnce(&mut Self) -> O>(entity: &mut Entity, f: F) -> Option<O>;
}

macro_rules! define_entity {
    (
        $( $propname:ident : $propt:path),* ;
        $( $name:ident => $t:path),* $(,)*
    ) => {
        #[derive(Debug)]
        pub struct Entity {
            $(
                pub $propname : $propt,
            )*
            $(
                pub $name: Option<Box<$t>>,
            )*
        }

        $(
            impl Component for $t {
                type Entity = Entity;
                #[inline]
                fn set(self, entity: &mut Entity) {
                    entity.$name = Some(Box::new(self))
                }

                #[inline]
                fn get(entity: &Entity) -> Option<&$t> {
                    entity.$name.as_ref().map(|s| &**s)
                }

                #[inline]
                fn get_mut(entity: &mut Entity) -> Option<&mut $t> {
                    entity.$name.as_mut().map(|s| &mut **s)
                }

                #[inline]
                fn remove(entity: &mut Entity) -> Option<Box<$t>> {
                    entity.$name.take()
                }

                #[inline]
                fn peek<O, F: FnOnce(&Self) -> O>(entity: &Entity, f: F) -> Option<O> {
                    entity.$name.as_ref().map(|c| &**c).map(f)
                }

                #[inline]
                fn update<O, F: FnOnce(&mut Self) -> O>(entity: &mut Entity, f: F) -> Option<O> {
                    entity.$name.as_mut().map(|c| &mut **c).map(f)
                }
            }
        )*

        impl EntityBase for Entity {
            type CreationParams = ( $( $propt ),* );

            fn new( ( $( $propname ),* ) : ( $( $propt ),*) ) -> Self {
                Entity {
                    $(
                        $propname: $propname,
                    )*
                    $(
                        $name: None
                    )*
                }
            }

            fn for_each_active_component(&self, mut f: impl FnMut(TypeId)) {
                $(
                    if let Some(_p) = &self.$name {
                        f(TypeId::of::< $t >())
                    };
                )*
            }

            fn for_all_components(mut f: impl FnMut(TypeId)) {
                // todo, replace this by const once TypeId::of is a const fn
                let components_type_ids: &[TypeId] = &[$( TypeId::of::<$t>() ),*];
                for component_id in components_type_ids {
                    f(*component_id);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct A {}
#[derive(Debug)]
pub struct A2 {}
#[derive(Debug)]
pub struct B {}

define_entity!{ a: A, a2: A2;
    b => B,
}

pub trait EntityBase: Sized + 'static {
    type CreationParams;

    fn new(params: Self::CreationParams) -> Self;

    // For a specific entity, go through every component this entity has.
    fn for_each_active_component(&self, f: impl FnMut(TypeId));

    // Go through all possible components this kind of entity might have.
    fn for_all_components(f: impl FnMut(TypeId));
}
//     #[inline]
//     fn with<C: Component>(mut self, component: C) -> Self where C::Entity = Self {
//         component.set(&mut self);
//         self
//     }

//     #[inline]
//     fn with_mutation<C: Component, F: FnOnce(&mut C)>(mut self, f: F) -> Self {
//         self.mutate(f);
//         self
//     }

//     #[inline]
//     fn peek<C: Component, F: FnOnce(&C)>(&self, f: F) {
//         if let Some(r) = self.get::<C>() {
//             f(r)
//         }
//     }

//     #[inline]
//     fn mutate<C: Component, F: FnOnce(&mut C)>(&mut self, f: F) {
//         if let Some(r) = self.get_mut::<C>() {
//             f(r)
//         }
//     }

//     #[inline]
//     fn has<C: Component>(&self) -> bool {
//         C::get(self).is_some()
//     }

//     #[inline]
//     fn get<C: Component>(&self) -> Option<&C> {
//         C::get(self)
//     }

//     #[inline]
//     fn get_mut<C: Component>(&mut self) -> Option<&mut C> {
//         C::get_mut(self)
//     }

//     #[inline]
//     fn remove<C: Component>(&mut self) -> Option<Box<C>> {
//         C::remove(self)
//     }
// }