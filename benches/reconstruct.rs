
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use mobec::{EntityList, EntityBase, define_entity};

use generational_arena::Arena;

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
    #[derive(Debug)]
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

fn generate_dual_component_arena(list_size: u32) -> Arena<Entity> {
    let mut arena: Arena<Entity> = Arena::new();

    let mut c: u32 = 0;
    let mut is_static = true;

    for i in 0..(list_size / 10) {
        for j in 0..5 {
            if c == 0 {
                arena.insert(
                    Entity::new((P { x: j as f32, y: i as f32 },))
                        .with(CollisionBox { origin_x: -1.0, origin_y: -1.0, w: 2.0, h: 2.0, is_static })
                );
                arena.insert(
                    Entity::new((P { x: -(j as f32), y: -(i as f32) },))
                        .with(Speed { x: i as f32, y: 2.0* (j as f32) })
                        .with(CollisionBox { origin_x: -1.0, origin_y: -2.0, w: 4.0, h: 2.0, is_static })
                );
            } else {
                c = (c + 1) % 3;
                arena.insert(Entity::new((P { x: j as f32, y: i as f32 },)));
                arena.insert(
                    Entity::new((P { x: -(j as f32), y: -(i as f32) },))
                        .with(Speed { x: i as f32, y: 2.0* (j as f32) })
                );
            }
            is_static = !is_static;
        }
    }

    arena
}

fn reconstruct_bitsets(arena: Arena<Entity>) -> EntityList<Entity> {
    EntityList::from_arena(arena)
}

pub fn reconstruct_basic(c: &mut Criterion) {
    let mut group = c.benchmark_group("reconstruct bitsets");
    for size in [100, 200, 500, 1_000, 2_000, 5_000, 10_000, 20_000, 50_000, 100_000, 200_000, 500_000, 1_000_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched(|| generate_dual_component_arena(size as u32), |arena| reconstruct_bitsets(arena), criterion::BatchSize::LargeInput)
        });
    }
}

criterion_group!{
    name = benches;
    config = Criterion::default().sample_size(30);
    targets = reconstruct_basic
}
criterion_main!{benches}