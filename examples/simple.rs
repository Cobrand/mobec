use mobec::{define_entity, EntityList, EntityBase};

#[derive(Clone, Debug)]
pub struct A {
    n: i32
}
#[derive(Clone, Debug)]
pub struct B {
    b: String
}
#[derive(Debug)]
pub struct C {
    c: bool
}

impl Clone for C {
    fn clone(&self) -> Self {
        Self {
            c: self.c
        }
    }

    fn clone_from(&mut self, other: &Self) {
        println!("C::clone_from called");
        self.c.clone_from(&other.c);
    }
}

// define an entity `Entity` with the properties a: A, and the components b: B
// even at runtime, NO OTHER COMPONENT can be added to this list. The list of components an
// entity can have is defined at compile time.
define_entity!{
    #[derive(Debug)]
    pub struct Entity {
        // if you have no props, use `props => {}` instead.
        props => { a: A }
        components => {
            b => B,
            c => C,
        }
    }
}

fn main() {
    // creates an empty list for the entity we defined above.
    let mut entity_list: EntityList<Entity> = EntityList::new();

    // add an entity with the Component B initialized, and retrieve its id.
    let id_1 = entity_list.insert(
        // Entity::new takes as arguments the properties as tuple in order.
        // For instance,
        // for no properties, the empty tuple is expected,
        // for a single property A, the param is (A,)
        // for a two properties A and B, the param is (A, B)
        // and so on
        Entity::new((A { n: 0},))
            .with(B { b: String::from("hello world") })
    );

    // add an entity with no components
    let id_2 = entity_list.insert(
        Entity::new((A {n: 1 },))
    );

    let _id_3 = entity_list.insert(
        Entity::new((A {n: 5 },))
            .with(C { c: false })
    );

    for (_id, entity) in entity_list.iter_all_mut() {
        // loop through ALL the entities.

        // access a property by simply using a member access:
        entity.a.n += 1;
        
        // access a component also by using a member access, but the type is different,
        // so there are methods available to help you for common operations on an Entity's components.
        let _component_b: &mut Option<Box<B>> = &mut entity.b;
    };

    for (_id, entity) in entity_list.iter_mut::<(B,)>() {
        // loop mutably through the entities which have the component B.
        // even though we only have 1 element, we must use ::<(B,)> and not ::<B>
        // WARNING: we must not change the components of the entities there, otherwise the bitset cache
        // would be invalidated.

        // this method is part of the trait `EntityBase`, which you should import.
        // EntityBase has quite a few methods that will help you observe/mutate a component from
        // an entity.
        entity.mutate(|component_b: &mut B| {
            // mutates the component B only if the entity already has the component B.
            component_b.b = String::from("goodbye world")
        });
    };
    
    for (_id, _entity) in entity_list.iter_mut::<(B, C)>() {
        // loop mutably through the entities which have the components B AND C.
    };

    // retrieve the component B of the first entity.
    let b = entity_list.get(id_1).and_then(Entity::get::<B>);
    println!("id1: {:?}", b); // prints "Some( B { b: "goodbye world" })""
    
    // retrieve the component B of the second entity.
    let b = entity_list.get(id_2).and_then(Entity::get::<B>);
    println!("id2: {:?}", b); // prints "None"

    if let Some(_e) = entity_list.remove(id_2) {
        // remove the entity from the list, and retrieve the deleted entity.
    }

    let mut entity_list2 = entity_list.clone();
    entity_list2.clone_from(&entity_list);
}