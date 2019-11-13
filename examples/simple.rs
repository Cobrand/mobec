use rubyec::{define_entity, EntityList, EntityBase};

#[derive(Debug)]
pub struct A {
    i: i32
}
#[derive(Debug)]
pub struct B {
    b: String
}

define_entity!{ a: A;
    b => B,
}

fn main() {
    let mut entity_list: EntityList<Entity> = EntityList::new();

    entity_list.add_bitset_for_component::<B>();

    let id_1 = entity_list.insert(
        Entity::new((A { i: 0},))
            .with(B { b: String::from("hello world") })
    );
    let id_2 = entity_list.insert(
        Entity::new((A {i: 1 },))
    );

    let b = entity_list.get(id_1).and_then(Entity::get::<B>);
    println!("id1: {:?}", b); // prints "Some( B { b: "hello world" })""
    
    let b = entity_list.get(id_2).and_then(Entity::get::<B>);
    println!("id2: {:?}", b); // prints "None"
}