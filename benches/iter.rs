use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
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

define_entity!{
    pub struct Entity {
        props => {
            pos: P,
        },
        components => {
            speed => Speed,
            collision_box => CollisionBox,
        }
    }
}

fn generate_single_list(list_size: u32) -> EntityList<Entity> {
    let mut entity_list: EntityList<Entity> = EntityList::new();

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
    for (_i, e) in list.iter_mut::<(Speed,)>() {
        let Speed {x: speed_x, y: speed_y } = e.speed.as_ref().unwrap().as_ref();
        e.pos.x += speed_x;
        e.pos.y += speed_y;
    }
}

fn generate_dual_component_list_packed(list_size: u32) -> EntityList<Entity> {
    let mut entity_list: EntityList<Entity> = EntityList::new();

    let mut is_static = true;

    for i in 0..list_size {
        if i >= list_size - 100 {
            entity_list.insert(
                Entity::new((P { x: -(i as f32), y: -(i as f32) },))
                    .with(Speed { x: i as f32, y: 2.0* (i as f32) })
                    .with(CollisionBox { origin_x: -1.0, origin_y: -2.0, w: 4.0, h: 2.0, is_static })
            );
        } else {
            entity_list.insert(
                Entity::new((P { x: i as f32, y: i as f32 },))
                    .with(CollisionBox { origin_x: -1.0, origin_y: -1.0, w: 2.0, h: 2.0, is_static })
            );
        }
        is_static = !is_static;
    }

    entity_list
}

fn generate_dual_component_list_group(list_size: u32) -> EntityList<Entity> {
    let mut entity_list: EntityList<Entity> = EntityList::new();

    let mut c: u32 = 0;
    let mut is_static = true;

    for i in 0..list_size {
        if c <= 75 {
            entity_list.insert(
                Entity::new((P { x: -(i as f32), y: -(i as f32) },))
                    .with(Speed { x: i as f32, y: 2.0* (i as f32) })
                    .with(CollisionBox { origin_x: -1.0, origin_y: -2.0, w: 4.0, h: 2.0, is_static })
            );
        } else if c <= 85 {
            entity_list.insert(
                Entity::new((P { x: i as f32, y: i as f32 },))
                    .with(CollisionBox { origin_x: -1.0, origin_y: -1.0, w: 2.0, h: 2.0, is_static })
            );
        } else if c <= 100 {
            entity_list.insert(
                Entity::new((P { x: -(i as f32), y: -(i as f32) },))
                    .with(Speed { x: i as f32, y: 2.0* (i as f32) })
            );
        } else {
            entity_list.insert(
                Entity::new((P { x: -(i as f32), y: -(i as f32) },))
            );
            if c > 200 {
                c = 0;
            }
        }
        c = c + 1;
        is_static = !is_static;
    }

    entity_list
}

fn generate_dual_component_list(list_size: u32) -> EntityList<Entity> {
    let mut entity_list: EntityList<Entity> = EntityList::new();

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

fn generate_dual_component_list_quite_sparse(list_size: u32) -> EntityList<Entity> {
    generate_dual_component_list_sparse(list_size, 19, 12)
}

fn generate_dual_component_list_much_sparse(list_size: u32) -> EntityList<Entity> {
    generate_dual_component_list_sparse(list_size, 89, 43)
}

fn generate_dual_component_list_sparse(list_size: u32, p1: u32, p2: u32) -> EntityList<Entity> {
    let mut entity_list: EntityList<Entity> = EntityList::new();

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
            c = (c + 1) % p1;
            s = (s + 1) % p2;

            entity_list.insert(e);
        }
    };

    entity_list
}

fn update_dual_component_list(list: &mut EntityList<Entity>) {
    for (_i, e) in list.iter_mut::<(Speed, CollisionBox,)>() {
        let Speed {x: speed_x, y: speed_y } = e.speed.as_ref().unwrap().as_ref();
        let c: &CollisionBox = e.collision_box.as_ref().unwrap().as_ref();
        if ! c.is_static {
            e.pos.x += speed_x;
            e.pos.y += speed_y;
        }
    }
}

fn maybe_update_dual_component_list(list: &mut EntityList<Entity>) {
    for (_i, e) in list.iter_all_mut() {
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
    let mut group = c.benchmark_group("single_component");
    for size in [100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut list = generate_single_list(size as u32);
            b.iter(|| update_single_list(&mut list))
        });
    }
}

pub fn iter_dual_component(c: &mut Criterion) {
    let mut group = c.benchmark_group("dual_component");
    for size in [100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut list = generate_dual_component_list(size as u32);
            b.iter(|| update_dual_component_list(&mut list))
        });
    }
}

pub fn iter_dual_component_sparse(c: &mut Criterion) {
    let mut group = c.benchmark_group("dual_component_sparse1");
    for size in [100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut list = generate_dual_component_list_quite_sparse(size as u32);
            b.iter(|| update_dual_component_list(&mut list))
        });
    }
}

pub fn iter_dual_component_very_sparse(c: &mut Criterion) {
    let mut group = c.benchmark_group("dual_component_sparse2");
    for size in [100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut list = generate_dual_component_list_much_sparse(size as u32);
            b.iter(|| update_dual_component_list(&mut list))
        });
    }
}

pub fn iter_dual_component_grouped(c: &mut Criterion) {
    let mut group = c.benchmark_group("dual_component_grouped");
    for size in [100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut list = generate_dual_component_list_group(size as u32);
            b.iter(|| update_dual_component_list(&mut list))
        });
    }
}

pub fn iter_dual_component_packed(c: &mut Criterion) {
    let mut group = c.benchmark_group("dual_component_packed");
    for size in [100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut list = generate_dual_component_list_packed(size as u32);
            b.iter(|| update_dual_component_list(&mut list))
        });
    }
}

pub fn iter_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("iter_all");
    for size in [100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut list = generate_dual_component_list_quite_sparse(size as u32);
            b.iter(|| maybe_update_dual_component_list(&mut list))
        });
    }
}

criterion_group!{
    name = benches;
    config = Criterion::default().sample_size(30);
    targets = iter_single_component, iter_dual_component, iter_dual_component_sparse, iter_dual_component_very_sparse, iter_dual_component_grouped, iter_dual_component_packed, iter_all
}
criterion_main!{benches}