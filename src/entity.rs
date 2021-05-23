
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
///
/// An entity has two main members:
///
/// * Properties, which are mandatory members on all your entities. Example: a position.
/// * Components, which are optional members taht may be added or removed at runtime. Examples:
/// a speed, a body, ...
///
/// The code below:
///
/// ```ignore
/// define_entity!{
///     #[derive(Debug)]
///     pub struct Entity {
///         // if you have no props, use `props => {}` instead.
///         props => { a: A }
///         components => {
///             b => B,
///             c => C,
///         }
///     }
/// }
/// ```
///
/// will roughly generate the following code:
///
/// ```ignore
/// #[derive(Debug)]
/// pub struct Entity {
///     pub a: A,
///     pub b: Option<Box<B>>,
///     pub c: Option<Box<C>>,
/// }
///
/// impl EntityBase for Entity { ... }
///
/// impl Component<Entity> for B { ... }
/// impl Component<Entity> for C { ... }
/// ```
///
/// Even if your components and your entity don't derive Debug, you must have a `#[derive()]`
/// attribute, even if it is empty. Likewise, even if you have to properties or no components,
/// the arm must be there, they just have to be empty.
///
/// ```rust
/// # use mobec::define_entity;
/// define_entity! {
///     #[derive()]
///     pub struct Entity {
///         props => {},
///         components => {}
///     }
/// }
/// ```
///
/// You can derive just as many things as you'd like with a regular struct. Only `Copy` is forbidden
/// if using components. Example:
///
/// ```ignore
/// define_entity! {
///     #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize)]
///     pub struct Entity {
///         props => {},
///         components => {}
///     }
/// }
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
            impl mobec::Component<$entityname> for $componenttype {
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

        impl Clone for $entityname {
            fn clone(&self) -> Self {
                Self {
                    $(
                        $propname: self.$propname.clone(),
                    )*
                    $(
                        $componentname: self.$componentname.clone(),
                    )*
                }
            }

            fn clone_from(&mut self, other: &Self) {
                $(
                    self.$propname.clone_from(&other.$propname);
                )*
                $(
                    self.$componentname.clone_from(&other.$componentname);
                )*
            }
        }

        impl mobec::EntityBase for $entityname {
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
                    if self.$componentname.is_some() {
                        f(std::any::TypeId::of::< $componenttype >())
                    };
                )*
            }

            fn for_each_component(&self, mut f: impl FnMut(std::any::TypeId, bool)) {
                $(
                    f(std::any::TypeId::of::< $componenttype >(), self.$componentname.is_some());
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
    /// CreationParams are always the properties of an entity.
    type CreationParams;

    /// Creates an entity with the given properties.
    ///
    /// Entity::new takes as arguments the properties as tuple in order.
    ///
    /// For instance:
    /// * for no properties, the empty tuple is expected,
    /// * for a single property A, the param is (A,)
    /// * for a two properties A and B, the param is (A, B)
    /// * and so on
    fn new(params: Self::CreationParams) -> Self;

    // For a specific entity, go through every component this entity has.
    fn for_each_active_component(&self, f: impl FnMut(TypeId));

    // For a specific entity, go through every component this entity may have. A boolean
    // is attached to know whether the component is actually there or not.
    fn for_each_component(&self, f: impl FnMut(TypeId, bool));

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
    ///
    /// # Example
    ///
    /// ```ignore
    /// let i: i32 = 4;
    /// let e = e.with_component_change(|e: &mut Entity| -> ChangeComponent<ComponentA> {
    ///     if i % 2 == 0 {
    ///         let beta = i + 1;
    ///         ChangeComponent::Mutate(Box::new(move |a: &mut ComponentA| {
    ///             a.alpha += beta as f32;
    ///         }))
    ///     } else {
    ///         ChangeComponent::NoChange
    ///     }
    /// });
    /// ```
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

    #[inline]
    /// Remove a component from the given entity.
    ///
    /// If the entity had the requested component, it is returned.
    fn add<C: Component<Self>>(&mut self, c: C) {
        c.set(self);
    }
}