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

fn generate_dual_component_list_sparse(list_size: u32) -> EntityList<Entity> {
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
            c = (c + 1) % 5;
            s = (s + 1) % 7;

            entity_list.insert(e);
        }
    };

    entity_list
}

fn main() {
    let mut list = generate_dual_component_list_sparse(100);
    update_dual_component_list(&mut list);
    println!("length: {}", list.len());
}