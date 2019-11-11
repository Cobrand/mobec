use rubyec::{define_entity, EntityBase};

#[derive(Debug)]
pub struct A {
    i: i32
}
#[derive(Debug)]
pub struct A2 {
    j: f32
}
#[derive(Debug)]
pub struct B {
    b: String
}

define_entity!{ a: A;
    b => B,
}

fn main() {
    let entity = Entity::new(A { i: 0} )
        .with(B { b: String::new() });
}