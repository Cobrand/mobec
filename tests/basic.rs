use rubyec::{
    define_entity,
    EntityList,
    EntityBase,
};

#[derive(Debug, PartialEq)]
pub struct ComponentA {
    alpha: f32,
}

#[derive(Debug, PartialEq)]
pub struct ComponentB {
    beta: i32,
}

#[derive(Debug, PartialEq)]
pub struct ComponentC {
    ceta: u32,
}

#[derive(Debug, PartialEq)]
pub struct CommonProp;

#[derive(Debug, PartialEq)]
pub struct AgeProp {
    age: u32,
}

define_entity! { common: CommonProp, age: AgeProp ;
    a => ComponentA,
    b => ComponentB,
    c => ComponentC,
}

// fn generate_sample_data() -> EntityList<Entity> {
//     let mut entity_list: EntityList<Entity> = EntityList::new();

//     entity_list.add_bitset_for_component::<ComponentA>();
//     entity_list.add_bitset_for_component::<ComponentB>();

//     entity_list.insert(
//         Entity::new((CommonProp, AgeProp { age: 5 }))
//             .with(ComponentA { alpha: 5.0 })
//     );
//     entity_list.insert(
//         Entity::new((CommonProp, AgeProp { age: 6 }))
//             .with(ComponentB { beta: 5 })
//     );
//     entity_list.insert(
//         Entity::new((CommonProp, AgeProp { age: 7 }))
//             .with(ComponentA { alpha: 5.0 })
//             .with(ComponentB { beta: 5 })
//     );
//     entity_list.insert(
//         Entity::new((CommonProp, AgeProp { age: 8 }))
//             .with(ComponentC { ceta: 5 })
//     );
//     entity_list.insert(
//         Entity::new((CommonProp, AgeProp { age: 9 }))
//             .with(ComponentA { alpha: 5.0 })
//             .with(ComponentC { ceta: 5 })
//     );

//     entity_list
// }

#[test]
fn entity_ops() {
    let mut entity_list: EntityList<Entity> = EntityList::new();

    let id_1 = entity_list.insert(
        Entity::new((CommonProp, AgeProp { age: 5 }))
            .with(ComponentA { alpha: 5.0 })
    );
    let id_2 = entity_list.insert(
        Entity::new((CommonProp, AgeProp { age: 6 }))
            .with(ComponentB { beta: 5 })
    );
    let e1 = entity_list.get(id_1).unwrap();
    let x = e1.get::<ComponentA>().unwrap();
    debug_assert_eq!(x.alpha, 5.0);
    debug_assert_eq!(e1.get::<ComponentB>(), None);

    let e2 = entity_list.get_mut(id_2).unwrap();
    let x = e2.get_mut::<ComponentB>().unwrap();
    x.beta += 1;

    debug_assert!(e2.has::<ComponentB>());
    debug_assert!(! e2.has::<ComponentA>());
    e2.mutate(|c: &mut ComponentB| { c.beta += 1 });

    let v = e2.peek(|c: &ComponentB| c.beta );
    debug_assert_eq!(v, Some(7));

    e2.remove::<ComponentB>();

    debug_assert_eq!(e2.get::<ComponentB>(), None);
}

#[test]
fn iter() {
    let mut entity_list: EntityList<Entity> = EntityList::new();

    let id_1 = entity_list.insert(
        Entity::new((CommonProp, AgeProp { age: 5 }))
            .with(ComponentA { alpha: 5.0 })
    );
    let id_2 = entity_list.insert(
        Entity::new((CommonProp, AgeProp { age: 1 }))
            .with(ComponentB { beta: 5 })
    );
    let id_3 = entity_list.insert(
        Entity::new((CommonProp, AgeProp { age: 6 }))
            .with(ComponentB { beta: 6 })
            .with(ComponentA { alpha: 6.0 })
    );
    let id_4 = entity_list.insert(
        Entity::new((CommonProp, AgeProp { age: 6 }))
            .with(ComponentC { ceta: 6 })
    );
    let id_5 = entity_list.insert(
        Entity::new((CommonProp, AgeProp { age: 6 }))
            .with(ComponentB { beta: 6 })
            .with(ComponentC { ceta: 6 })
    );
    let id_6 = entity_list.insert(
        Entity::new((CommonProp, AgeProp { age: 6 }))
            .with(ComponentA { alpha: 6.0 })
            .with(ComponentB { beta: 6 })
            .with(ComponentC { ceta: 6 })
    );

    entity_list.add_bitset_for_component::<ComponentA>();
    entity_list.add_bitset_for_component::<ComponentB>();

    let all_entities: Vec<_> = entity_list.iter().map(|(i, _e)| i).collect();
    let only_comp_a: Vec<_> = entity_list.iter_for_components::<(ComponentA,)>().map(|(i, _e)| i).collect();
    let only_comp_b: Vec<_> = entity_list.iter_for_components::<(ComponentB,)>().map(|(i, _e)| i).collect();
    let only_comp_c: Vec<_> = entity_list.iter_for_components::<(ComponentC,)>().map(|(i, _e)| i).collect();
    let comp_a_and_b: Vec<_> = entity_list.iter_for_components::<(ComponentA, ComponentB)>().map(|(i, _e)| i).collect();
    let comp_a_and_c: Vec<_> = entity_list.iter_for_components::<(ComponentA, ComponentC)>().map(|(i, _e)| i).collect();
    let comp_b_and_c: Vec<_> = entity_list.iter_for_components::<(ComponentB, ComponentC)>().map(|(i, _e)| i).collect();
    let comp_all: Vec<_> = entity_list.iter_for_components::<(ComponentB, ComponentC, ComponentA)>().map(|(i, _e)| i).collect();

    debug_assert_eq!(all_entities, &[id_1, id_2, id_3, id_4, id_5, id_6]);

    debug_assert_eq!(only_comp_a, &[id_1, id_3, id_6]);
    debug_assert_eq!(only_comp_b, &[id_2, id_3, id_5, id_6]);
    debug_assert_eq!(only_comp_c, &[id_4, id_5, id_6]);

    debug_assert_eq!(comp_a_and_b, &[id_3, id_6]);
    debug_assert_eq!(comp_a_and_c, &[id_6]);
    debug_assert_eq!(comp_b_and_c, &[id_5, id_6]);
    
    debug_assert_eq!(comp_all, &[id_6]);
}