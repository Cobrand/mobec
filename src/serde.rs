use crate::{EntityList, EntityBase};

use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

use generational_arena::Arena;

impl<E> Serialize for EntityList<E>
where
    E: Serialize + EntityBase,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.entities.serialize(serializer)
    }
}

impl<'de, E> Deserialize<'de> for EntityList<E> where E: Deserialize<'de> + EntityBase {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let arena: Arena<E> = Deserialize::deserialize(deserializer)?;
        Ok(EntityList::from_arena(arena))
    }
}