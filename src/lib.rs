use std::any::TypeId;

pub trait Component<E: Sized>: 'static {
    fn set(self, entity: &mut E);

    fn get(entity: &E) -> Option<&Self>;

    fn get_mut(entity: &mut E) -> Option<&mut Self>;

    /// Delete a component from an entity
    fn remove(entity: &mut E) -> Option<Box<Self>>;
    
    // read a component with the given predicate. You may return a custom result of your choice.
    fn peek<O, F: FnOnce(&Self) -> O>(entity: &E, f: F) -> Option<O>;

    // update component with the given predicate. You may return a custom result of your choice.
    fn update<O, F: FnOnce(&mut Self) -> O>(entity: &mut E, f: F) -> Option<O>;
}

#[macro_export]
macro_rules! define_entity {
    (
        $( $propname:ident : $propt:path),* ;
        $( $name:ident => $t:path),* $(,)*
    ) => {
        use std::any::TypeId;

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
            impl Component<Entity> for $t {
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

pub enum ChangeComponent<C> {
    /// Do not change the given component
    NoChange,
    /// Replace the given component by a new one. Works even if there was no component to begin with.
    Replace(C),
    /// Mutate the currently available component. Only works if there is a component to begin with.
    Mutate(Box<dyn FnOnce(&mut C)>),
    /// Remove the component without adding a new one.
    Remove,
}

pub trait EntityBase: Sized + 'static {
    type CreationParams;

    fn new(params: Self::CreationParams) -> Self;

    // For a specific entity, go through every component this entity has.
    fn for_each_active_component(&self, f: impl FnMut(TypeId));

    // Go through all possible components this kind of entity might have.
    fn for_all_components(f: impl FnMut(TypeId));

    #[inline]
    /// Returns the ntity with the specified component. The old component is discarded.
    fn with<C: Component<Self>>(mut self, component: C) -> Self {
        component.set(&mut self);
        self
    }

    #[inline]
    /// Mutates the component for the given entity.
    ///
    /// Mutations only apply to inner changes, not removal or creation of components. The predicate
    /// is only called if the component exists for the given entity to begin with.
    fn with_mutation<C: Component<Self>, F: FnOnce(&mut C)>(mut self, f: F) -> Self {
        self.mutate(f);
        self
    }

    #[inline]
    /// Removes the given component for the given entity.
    ///
    /// Mutations only apply to inner changes, not removal or creation of components. The predicate
    /// is only called if the component exists for the given entity to begin with.
    fn with_removed<C: Component<Self>>(mut self) -> Self {
        self.remove::<C>();
        self
    }

    #[inline]
    fn with_component_change<'a, C: Component<Self>, F: FnOnce(&mut Self) -> ChangeComponent<C>>(mut self, f: F) -> Self {
        match f(&mut self) {
            ChangeComponent::NoChange => self,
            ChangeComponent::Remove => self.with_removed::<C>(),
            ChangeComponent::Replace(c) => self.with(c),
            ChangeComponent::Mutate(f) => {
                if let Some(c) = self.get_mut::<C>() {
                    f(c)
                };
                self
            },
        }
    }

    #[inline]
    /// Peek the properties of the given component type, for the given entity, using the given predicate.
    fn peek<C: Component<Self>, F: FnOnce(&C)>(&self, f: F) {
        if let Some(r) = self.get::<C>() {
            f(r)
        }
    }

    #[inline]
    /// Mutate the properties of the given component type, for the given entity, using the given predicate.
    fn mutate<C: Component<Self>, F: FnOnce(&mut C)>(&mut self, f: F) {
        if let Some(r) = self.get_mut::<C>() {
            f(r)
        }
    }

    #[inline]
    /// Returns true if the entity has the requested component type as an active component.
    fn has<C: Component<Self>>(&self) -> bool {
        C::get(self).is_some()
    }

    #[inline]
    fn get<C: Component<Self>>(&self) -> Option<&C> {
        C::get(self)
    }

    #[inline]
    fn get_mut<C: Component<Self>>(&mut self) -> Option<&mut C> {
        C::get_mut(self)
    }

    #[inline]
    /// Remove a component from the given entity.
    ///
    /// If the entity had the requested component, it is returned.
    fn remove<C: Component<Self>>(&mut self) -> Option<Box<C>> {
        C::remove(self)
    }
}