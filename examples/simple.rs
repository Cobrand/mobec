use rubyec::*;

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

define_entity!{ a: A, a2: A2;
    b => B,
}

fn main() {
    let entity = Entity::new((A { i: 0} , A2 {j: 0.0}));
}