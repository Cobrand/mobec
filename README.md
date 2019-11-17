
MobEC - **Mob** **E**ntity **C**omponent library for Rust.

Unlike other ECS libraries, mobec does not specifify the S of ECS. This is simply
because there is nothing helping you on the "system" side of ECS. You may use systems,
or any other method to handle logic for all that matters, this library only takes care
of linking Entities and Components.

To get started, have a look a the macro `define_entity`, which will help you define your
entity, or `EntityList` which will allow you to have a list that holds multiple entities.

The name "mob" comes from the fact that this library is extremely basic and has very few features
whatsoever, like the character of the same name.

# Features

* Backed by a generational-arena and a hibitset for caching
* Optional serde integration (`use_serde` feature).
* Made to be flexible with your systems, but strict with your components.
* Speed is not to be ignored, but it not the priority: every entity has an `Option<Box<_>>` for component.
This is fine if you want to process less than 10000~100000 entities at a time. More than that,
and you might want to use faster options like [specs](https://crates.io/crates/specs). Keep in mind that this is a trade of usability/speed.