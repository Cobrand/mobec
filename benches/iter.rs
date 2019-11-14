use criterion::{criterion_group, criterion_main, Criterion};
use rubyec::{EntityList, EntityBase, define_entity};

#[derive(Debug, Clone, Copy)]
pub struct P { 
    x: f32,
    y: f32
}

#[derive(Debug, Clone, Copy)]
pub struct Speed {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct CollisionBox {
    origin_x: f32,
    origin_y: f32,
    w: f32,
    h: f32,
    is_static: bool,
}

define_entity!{ pos: P;
    speed => Speed,
    collision_box => CollisionBox,
}

fn generate_single_list(list_size: u32) -> EntityList<Entity> {
    let mut entity_list: EntityList<Entity> = EntityList::new();

    entity_list.add_bitset_for_component::<Speed>();

    for i in 0..(list_size / 10) {
        for j in 0..5 {
            entity_list.insert(Entity::new((P { x: j as f32, y: i as f32 },)));
            entity_list.insert(
                Entity::new((P { x: -(j as f32), y: -(i as f32) },))
                    .with(Speed { x: i as f32, y: 2.0* (j as f32) })
            );
        }
    }

    entity_list
}

fn update_single_list(list: &mut EntityList<Entity>) {
    for (_i, e) in list.iter_mut_for_components::<(Speed,)>() {
        let Speed {x: speed_x, y: speed_y } = e.speed.as_ref().unwrap().as_ref();
        e.pos.x += speed_x;
        e.pos.y += speed_y;
    }
}

fn generate_dual_component_list(list_size: u32) -> EntityList<Entity> {
    let mut entity_list: EntityList<Entity> = EntityList::new();

    entity_list.add_bitset_for_component::<Speed>();
    entity_list.add_bitset_for_component::<CollisionBox>();

    let mut c: u32 = 0;
    let mut is_static = true;

    for i in 0..(list_size / 10) {
        for j in 0..5 {
            if c == 0 {
                entity_list.insert(
                    Entity::new((P { x: j as f32, y: i as f32 },))
                        .with(CollisionBox { origin_x: -1.0, origin_y: -1.0, w: 2.0, h: 2.0, is_static })
                );
                entity_list.insert(
                    Entity::new((P { x: -(j as f32), y: -(i as f32) },))
                        .with(Speed { x: i as f32, y: 2.0* (j as f32) })
                        .with(CollisionBox { origin_x: -1.0, origin_y: -2.0, w: 4.0, h: 2.0, is_static })
                );
            } else {
                c = (c + 1) % 3;
                entity_list.insert(Entity::new((P { x: j as f32, y: i as f32 },)));
                entity_list.insert(
                    Entity::new((P { x: -(j as f32), y: -(i as f32) },))
                        .with(Speed { x: i as f32, y: 2.0* (j as f32) })
                );
            }
            is_static = !is_static;
        }
    }

    entity_list
}

fn generate_dual_component_list_sparse(list_size: u32) -> EntityList<Entity> {
    let mut entity_list: EntityList<Entity> = EntityList::new();

    entity_list.add_bitset_for_component::<Speed>();
    entity_list.add_bitset_for_component::<CollisionBox>();

    let mut s: u32 = 0;
    let mut c: u32 = 0;
    let mut is_static = true;

    for i in 0..(list_size / 10) {
        for j in 0..10 {
            let mut e = Entity::new((P { x: j as f32, y: i as f32 },));
            if c == 0 {
                e = e.with(CollisionBox { origin_x: -1.0, origin_y: -1.0, w: 2.0, h: 2.0, is_static })
            }
            if s == 0 {
                e = e.with(Speed { x: i as f32, y: 2.0* (j as f32) })
            }
            is_static = !is_static;
            c = (c + 1) % 19;
            s = (s + 1) % 21;

            entity_list.insert(e);
        }
    };

    entity_list
}

fn generate_dual_component_list_no_bitset(list_size: u32) -> EntityList<Entity> {
    let mut l = generate_dual_component_list(list_size);
    l.remove_bitset_for_component::<Speed>();
    l.remove_bitset_for_component::<CollisionBox>();
    l
}

fn generate_dual_component_list_partial_bitset(list_size: u32) -> EntityList<Entity> {
    let mut l = generate_dual_component_list(list_size);
    l.remove_bitset_for_component::<Speed>();
    l
}

fn update_dual_component_list(list: &mut EntityList<Entity>) {
    for (_i, e) in list.iter_mut_for_components::<(Speed, CollisionBox,)>() {
        let Speed {x: speed_x, y: speed_y } = e.speed.as_ref().unwrap().as_ref();
        let c: &CollisionBox = e.collision_box.as_ref().unwrap().as_ref();
        if ! c.is_static {
            e.pos.x += speed_x;
            e.pos.y += speed_y;
        }
    }
}

fn maybe_update_dual_component_list(list: &mut EntityList<Entity>) {
    for (_i, e) in list.iter_mut() {
        if e.has::<Speed>() && e.has::<CollisionBox>() {
            let Speed {x: speed_x, y: speed_y } = e.speed.as_ref().unwrap().as_ref();
            let c: &CollisionBox = e.collision_box.as_ref().unwrap().as_ref();
            if ! c.is_static {
                e.pos.x += speed_x;
                e.pos.y += speed_y;
            }
        }
    }
}

pub fn iter_single_component(c: &mut Criterion) {
    let mut list_100 = generate_single_list(100);
    c.bench_function("iter single component 100", |b| b.iter(|| update_single_list(&mut list_100)));
    let mut list_1_000 = generate_single_list(1000);
    c.bench_function("iter single component 1_000", |b| b.iter(|| update_single_list(&mut list_1_000)));
    let mut list_10_000 = generate_single_list(10_000);
    c.bench_function("iter single component 10_000", |b| b.iter(|| update_single_list(&mut list_10_000)));
    let mut list_100_000 = generate_single_list(100_000);
    c.bench_function("iter single component 100_000", |b| b.iter(|| update_single_list(&mut list_100_000)));
}

pub fn iter_dual_component(c: &mut Criterion) {
    let mut list_100 = generate_dual_component_list(100);
    c.bench_function("iter_dual_component_100", |b| b.iter(|| update_dual_component_list(&mut list_100)));
    let mut list_1_000 = generate_dual_component_list(1000);
    c.bench_function("iter_dual_component_1_000", |b| b.iter(|| update_dual_component_list(&mut list_1_000)));
    let mut list_10_000 = generate_dual_component_list(10_000);
    c.bench_function("iter_dual_component_10_000", |b| b.iter(|| update_dual_component_list(&mut list_10_000)));
    let mut list_100_000 = generate_dual_component_list(100_000);
    c.bench_function("iter_dual_component_100_000", |b| b.iter(|| update_dual_component_list(&mut list_100_000)));
}

pub fn iter_dual_component_sparse(c: &mut Criterion) {
    let mut list_100 = generate_dual_component_list_sparse(100);
    c.bench_function("iter_dual_component_sparse_100", |b| b.iter(|| update_dual_component_list(&mut list_100)));
    let mut list_1_000 = generate_dual_component_list(1000);
    c.bench_function("iter_dual_component_sparse_1_000", |b| b.iter(|| update_dual_component_list(&mut list_1_000)));
    let mut list_10_000 = generate_dual_component_list(10_000);
    c.bench_function("iter_dual_component_sparse_10_000", |b| b.iter(|| update_dual_component_list(&mut list_10_000)));
    let mut list_100_000 = generate_dual_component_list(100_000);
    c.bench_function("iter_dual_component_sparse_100_000", |b| b.iter(|| update_dual_component_list(&mut list_100_000)));
}

pub fn iter_dual_component_no_bitset(c: &mut Criterion) {
    let mut list_100 = generate_dual_component_list_no_bitset(100);
    c.bench_function("iter_dual_component_no_bitset_100", |b| b.iter(|| update_dual_component_list(&mut list_100)));
    let mut list_1_000 = generate_dual_component_list_no_bitset(1000);
    c.bench_function("iter_dual_component_no_bitset_1_000", |b| b.iter(|| update_dual_component_list(&mut list_1_000)));
    let mut list_10_000 = generate_dual_component_list_no_bitset(10_000);
    c.bench_function("iter_dual_component_no_bitset_10_000", |b| b.iter(|| update_dual_component_list(&mut list_10_000)));
    let mut list_100_000 = generate_dual_component_list_no_bitset(100_000);
    c.bench_function("iter_dual_component_no_bitset_100_000", |b| b.iter(|| update_dual_component_list(&mut list_100_000)));
}

pub fn iter_all_dual_component_no_bitset(c: &mut Criterion) {
    let mut list_100 = generate_dual_component_list_no_bitset(100);
    c.bench_function("iter_all_dual_component_no_bitset_100", |b| b.iter(|| maybe_update_dual_component_list(&mut list_100)));
    let mut list_1_000 = generate_dual_component_list_no_bitset(1000);
    c.bench_function("iter_all_dual_component_no_bitset_1_000", |b| b.iter(|| maybe_update_dual_component_list(&mut list_1_000)));
    let mut list_10_000 = generate_dual_component_list_no_bitset(10_000);
    c.bench_function("iter_all_dual_component_no_bitset_10_000", |b| b.iter(|| maybe_update_dual_component_list(&mut list_10_000)));
    let mut list_100_000 = generate_dual_component_list_no_bitset(100_000);
    c.bench_function("iter_all_dual_component_no_bitset_100_000", |b| b.iter(|| maybe_update_dual_component_list(&mut list_100_000)));
}

pub fn iter_dual_component_partial_bitset(c: &mut Criterion) {
    let mut list_100 = generate_dual_component_list_partial_bitset(100);
    c.bench_function("iter_dual_component_partial_bitset_100", |b| b.iter(|| update_dual_component_list(&mut list_100)));
    let mut list_1_000 = generate_dual_component_list_partial_bitset(1000);
    c.bench_function("iter_dual_component_partial_bitset_1_000", |b| b.iter(|| update_dual_component_list(&mut list_1_000)));
    let mut list_10_000 = generate_dual_component_list_partial_bitset(10_000);
    c.bench_function("iter_dual_component_partial_bitset_10_000", |b| b.iter(|| update_dual_component_list(&mut list_10_000)));
    let mut list_100_000 = generate_dual_component_list_partial_bitset(100_000);
    c.bench_function("iter_dual_component_partial_bitset_100_000", |b| b.iter(|| update_dual_component_list(&mut list_100_000)));
}

criterion_group!(benches,
    iter_single_component,
    iter_dual_component,
    iter_dual_component_sparse,
    iter_all_dual_component_no_bitset,
    iter_dual_component_no_bitset,
    iter_dual_component_partial_bitset
);
criterion_main!(benches);