#![deny(unsafe_code)]

//! mobec - **Mob** **E**ntity **C**omponent library
//!
//! Unlike other ECS libraries, mobec does not specifify the S of ECS. This is simply
//! because there is nothing helping you on the "system" side of ECS. You may use systems,
//! or any other method to handle logic for all that matters, this library only takes care
//! of linking Entities and Components.
//!
//! To get started, have a look a the macro [`define_entity`], which will help you define your
//! entity, or [`EntityList`] which will allow you to have a list that holds multiple entities.
//!
//! There is also a simple example below to help you get started.
//!
//! The name "mob" comes from the fact that this library is extremely basic and has very few features
//! whatsoever, like the character of the same name.
//!
//! ## Serde integration
//!
//! You will need to enable the feature `use_serde` of this crate.
//!
//! You can serialize and deserialize the [`EntityList`] if your entity implements `Serialize` and
//! `Deserialize`. You can either `#[derive(Serialize, Deserialize)]` your entity if all properties
//! and components also implement `Serialize` and `Deserialize`, or you can manually implement
//! both those traits for your entity.
//!
//! ## Components vs Properties
//!
//! Components are **optional** properties that you can add and remove at runtime. From a gamedev
//! perspective, examples include: `Position` (some entities may not have a position),
//! `Speed` (some entites may be static...), `CollisionBody` (some
//! entities may not collide at all, ...)
//!
//! Properties are members that are present on **every** entity no matter the entity. Often,
//! you will have 0 or 1 property for your entity, but you may have more depending on your needs.
//! Examples include `Position`, if every entity in your system has a position.
//!
//! [`define_entity`]: macro.define_entity.html
//! [`EntityList`]: struct.EntityList.html
//!
//! ```rust
//! use mobec::{define_entity, EntityList, EntityBase};
//! 
//! #[derive(Debug)]
//! pub struct A {
//!     n: i32
//! }
//! #[derive(Debug)]
//! pub struct B {
//!     b: String
//! }
//! #[derive(Debug)]
//! pub struct C {
//!     c: bool
//! }
//! 
//! // define an entity `Entity` with the properties a: A, and the components b: B
//! // even at runtime, NO OTHER COMPONENT can be added to this list. The list of components an
//! // entity can have is defined at compile time.
//! define_entity!{
//!     #[derive(Debug)]
//!     pub struct Entity {
//!         // if you have no props, use `props => {}` instead.
//!         props => { a: A }
//!         components => {
//!             b => B,
//!             c => C,
//!         }
//!     }
//! }
//! 
//! fn main() {
//!     // creates an empty list for the entity we defined above.
//!     let mut entity_list: EntityList<Entity> = EntityList::new();
//! 
//!     // add an entity with the Component B initialized, and retrieve its id.
//!     let id_1 = entity_list.insert(
//!         // Entity::new takes as arguments the properties as tuple in order.
//!         // For instance,
//!         // for no properties, the empty tuple is expected,
//!         // for a single property A, the param is (A,)
//!         // for a two properties A and B, the param is (A, B)
//!         // and so on
//!         Entity::new((A { n: 0},))
//!             .with(B { b: String::from("hello world") })
//!     );
//! 
//!     // add an entity with no components
//!     let id_2 = entity_list.insert(
//!         Entity::new((A {n: 1 },))
//!     );
//! 
//!     for (_id, entity) in entity_list.iter_all_mut() {
//!         // loop through ALL the entities.
//! 
//!         // access a property by simply using a member access:
//!         entity.a.n += 1;
//!         
//!         // access a component also by using a member access, but the type is different,
//!         // so there are methods available to help you for common operations on an Entity's components.
//!         let _component_b: &mut Option<Box<B>> = &mut entity.b;
//!     };
//! 
//!     for (_id, entity) in entity_list.iter_mut::<(B,)>() {
//!         // loop mutably through the entities which have the component B.
//!         // even though we only have 1 element, we must use ::<(B,)> and not ::<B>
//!         // WARNING: we must not change the components of the entities there, otherwise the bitset cache
//!         // would be invalidated.
//! 
//!         // this method is part of the trait `EntityBase`, which you should import.
//!         // EntityBase has quite a few methods that will help you observe/mutate a component from
//!         // an entity.
//!         entity.mutate(|component_b: &mut B| {
//!             // mutates the component B only if the entity already has the component B.
//!             component_b.b = String::from("goodbye world")
//!         });
//!     };
//!     
//!     for (_id, _entity) in entity_list.iter_mut::<(B, C)>() {
//!         // loop mutably through the entities which have the components B AND C.
//!     };
//! 
//!     // retrieve the component B of the first entity.
//!     let b = entity_list.get(id_1).and_then(Entity::get::<B>);
//!     println!("id1: {:?}", b); // prints "Some( B { b: "goodbye world" })""
//!     
//!     // retrieve the component B of the second entity.
//!     let b = entity_list.get(id_2).and_then(Entity::get::<B>);
//!     println!("id2: {:?}", b); // prints "None"
//! 
//!     if let Some(_e) = entity_list.remove(id_2) {
//!         // remove the entity from the list, and retrieve the deleted entity.
//!     }
//! }
//! ```

mod entity;
mod entity_list;
pub mod iter;

#[cfg(feature = "use_serde")]
mod serde;

pub use entity::*;
pub use entity_list::*;