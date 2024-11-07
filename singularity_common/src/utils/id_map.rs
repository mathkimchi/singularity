//! REVIEW: is this a good idea?
//! Even if I get the derives to work, this might be a bit wonky

use std::{collections::BTreeMap, marker::PhantomData};
use uuid::Uuid;

/// FIXME: derives
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Id<Item> {
    uuid: Uuid,
    /// Compiler will complain if I don't have this here
    #[serde(skip)]
    phantom_data: PhantomData<Item>,
}
impl<Item> Id<Item> {
    pub fn generate() -> Self {
        Uuid::new_v4().into()
    }
}
/// Derive doesn't understand phantomdata
mod id_derive_impls {
    use super::Id;
    use std::marker::PhantomData;
    use uuid::Uuid;

    impl<Item> From<Uuid> for Id<Item> {
        fn from(value: Uuid) -> Self {
            Self {
                uuid: value,
                phantom_data: PhantomData,
            }
        }
    }
    impl<Item> From<Id<Item>> for Uuid {
        fn from(value: Id<Item>) -> Self {
            value.uuid
        }
    }
    impl<Item> Copy for Id<Item> {}
    impl<Item> Clone for Id<Item> {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<Item> PartialEq for Id<Item> {
        fn eq(&self, other: &Self) -> bool {
            self.uuid.eq(&other.uuid)
        }
    }
    impl<Item> Eq for Id<Item> {}
    impl<Item> PartialOrd for Id<Item> {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }
    impl<Item> Ord for Id<Item> {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.uuid.cmp(&other.uuid)
        }
    }
    impl<Item> std::fmt::Debug for Id<Item> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Id").field("uuid", &self.uuid).finish()
        }
    }
}

pub type IdMap<Item> = BTreeMap<Id<Item>, Item>;
