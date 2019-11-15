
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

/// Macro to create an `Entity` type where this is called.
#[macro_export]
macro_rules! define_entity {
    (   #[derive( $( $derivety:path ),* ) ]
        $vis:vis struct $entityname:ident {
            props => {
                $( $propname:ident : $propt:ty),* $(,)*
            } $(,)?
            components => {
                $( $componentname:ident => $componenttype:ty ),* $(,)*
            } $(,)?
        }
    ) => {

        #[derive( $( $derivety ),* )]
        $vis struct $entityname {
            $(
                pub $propname : $propt,
            )*
            $(
                pub $componentname: Option<Box<$componenttype>>,
            )*
        }

        $(
            impl rubyec::Component<$entityname> for $componenttype {
                #[inline]
                fn set(self, entity: &mut $entityname) {
                    entity.$componentname = Some(Box::new(self))
                }

                #[inline]
                fn get(entity: &$entityname) -> Option<&$componenttype> {
                    entity.$componentname.as_ref().map(|s| &**s)
                }

                #[inline]
                fn get_mut(entity: &mut $entityname) -> Option<&mut $componenttype> {
                    entity.$componentname.as_mut().map(|s| &mut **s)
                }

                #[inline]
                fn remove(entity: &mut $entityname) -> Option<Box<$componenttype>> {
                    entity.$componentname.take()
                }

                #[inline]
                fn peek<O, F: FnOnce(&Self) -> O>(entity: &$entityname, f: F) -> Option<O> {
                    entity.$componentname.as_ref().map(|c| &**c).map(f)
                }

                #[inline]
                fn update<O, F: FnOnce(&mut Self) -> O>(entity: &mut $entityname, f: F) -> Option<O> {
                    entity.$componentname.as_mut().map(|c| &mut **c).map(f)
                }
            }
        )*

        impl rubyec::EntityBase for $entityname {
            type CreationParams = ( $( $propt ,)* );

            fn new( ( $( $propname ,)* ) : ( $( $propt ,)*) ) -> Self {
                $entityname {
                    $(
                        $propname: $propname,
                    )*
                    $(
                        $componentname: None,
                    )*
                }
            }

            fn for_each_active_component(&self, mut f: impl FnMut(std::any::TypeId)) {
                $(
                    if let Some(_p) = &self.$componentname {
                        f(std::any::TypeId::of::< $componenttype >())
                    };
                )*
            }

            fn for_all_components(mut f: impl FnMut(std::any::TypeId)) {
                // todo, replace this by const once TypeId::of is a const fn
                let components_type_ids: &[std::any::TypeId] = &[$( std::any::TypeId::of::<$componenttype>() ),*];
                for component_id in components_type_ids {
                    f(*component_id);
                }
            }
        }
    };
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
    fn with_removed<C: Component<Self>>(mut self) -> Self {
        self.remove::<C>();
        self
    }

    /// Depending on the current state of the component for the given entity, do some compelx operations.
    ///
    /// You must give a predicate that takes a `&mut Entity`, and returns a `ChangeComponent`.
    /// This is an enum that has four variants: one to change nothing, one to remove the component,
    /// one to replace (or add) a component, and another to mutate an already existing component.
    ///
    /// In all cases, the entity is returned. This is very useful if you have a component that is a "computed"
    /// value depending on other components.
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
    ///
    /// You may chosse to return a custom type in your predicate. If the entity has the component,
    /// your value is returned, otherwise `None` is returned.
    fn peek<C: Component<Self>, O, F: FnOnce(&C) -> O>(&self, f: F) -> Option<O> {
        self.get::<C>().map(f)
    }

    #[inline]
    /// Mutate the properties of the given component type, for the given entity, using the given predicate.
    ///
    /// You may choose to return a custom type in your predicate. If the entity has the component,
    /// your value is returned, otherwise `None` is returned.
    fn mutate<C: Component<Self>, O, F: FnOnce(&mut C) -> O>(&mut self, f: F) -> Option<O> {
        self.get_mut::<C>().map(f)
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