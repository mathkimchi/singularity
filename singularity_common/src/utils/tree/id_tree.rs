use super::tree_node_path::{TraversableTree, TreeNodePath};
use crate::utils::id_map::Id;
use std::collections::BTreeMap;

/// Technically, children are ordered
struct Node<T> {
    children: Vec<Id<T>>,
    parent: Option<Id<T>>,
}

/// Only stores hierarchy, no items.
pub struct IdTree<T> {
    root_id: Id<T>,
    nodes: BTreeMap<Id<T>, Node<T>>,
}
impl<T> IdTree<T> {
    pub fn new(root_id: Id<T>) -> Self {
        let mut nodes = BTreeMap::new();
        nodes.insert(root_id, Node::default());
        Self { root_id, nodes }
    }

    pub fn add_child(&mut self, parent_id: Id<T>, child_id: Id<T>) -> Option<()> {
        // add child to `nodes`
        self.nodes.insert(
            child_id,
            Node {
                children: Vec::new(),
                parent: Some(parent_id),
            },
        );
        // register child under parent
        self.nodes.get_mut(&parent_id)?.children.push(child_id);

        Some(())
    }

    /// generates a new id and adds it
    pub fn create_child(&mut self, parent_id: Id<T>) -> Option<Id<T>> {
        let child_id = Id::generate();
        self.add_child(parent_id, child_id);
        Some(child_id)
    }

    /// removes node corresponding to id and all its children
    ///
    /// If this is root, then don't remove
    ///
    /// TODO: do this with loop instead?
    pub fn remove_recursive(&mut self, id: Id<T>) -> bool {
        for child in self.nodes.get(&id).unwrap().children.clone() {
            self.remove_recursive(child);
        }

        if let Some(parent_id) = self.nodes.get(&id).unwrap().parent {
            // remove this from parent
            self.nodes
                .get_mut(&parent_id)
                .unwrap()
                .children
                .retain(|i| i != &id);
            // remove this from nodes
            self.nodes.remove(&id);

            true
        } else {
            // this is root
            println!("Tried to call remove on root");

            false
        }
    }

    pub fn get_root_id(&self) -> Id<T> {
        self.root_id
    }

    /// climb downwards
    pub fn get_id_from_path(&self, path: &TreeNodePath) -> Option<Id<T>> {
        let mut node_id = self.root_id;

        for index in path.0.iter() {
            node_id = *self.nodes.get(&node_id)?.children.get(*index)?;
        }

        Some(node_id)
    }

    pub fn get_children(&self, parent_id: Id<T>) -> &Vec<Id<T>> {
        &self.nodes.get(&parent_id).unwrap().children
    }

    /// climb upwards
    pub fn get_path(&self, id: Id<T>) -> Option<TreeNodePath> {
        let mut path_vec = Vec::new();
        let mut node_id = id;

        while let Some(parent_id) = self.nodes.get(&node_id).unwrap().parent {
            let child_index = self
                .nodes
                .get(&parent_id)
                .unwrap()
                .children
                .iter()
                .position(|id| id == &node_id)
                .unwrap();
            path_vec.push(child_index);

            node_id = parent_id;
        }

        path_vec.reverse();

        Some(path_vec.into())
    }
}
impl<T> TraversableTree for IdTree<T> {
    fn exists_at(&self, path: &TreeNodePath) -> bool {
        self.get_id_from_path(path).is_some()
    }
}

/// for some of the derive impls, I just took the type restrictions out of the derive macros
mod derive_macro_impls {
    use super::*;

    impl<T> Default for Node<T> {
        fn default() -> Self {
            Self {
                children: Default::default(),
                parent: Default::default(),
            }
        }
    }
    impl<T> Clone for Node<T> {
        fn clone(&self) -> Self {
            Self {
                children: self.children.clone(),
                parent: self.parent,
            }
        }
    }
    impl<T> serde::Serialize for Node<T> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            use serde::ser::SerializeStruct;
            let mut serialized = serializer.serialize_struct("Node", 2)?;
            serialized.serialize_field("children", &self.children)?;
            serialized.serialize_field("parent", &self.parent)?;
            serialized.end()
        }
    }
    /// Based off: https://serde.rs/deserialize-struct.html and by expanding the serialize macro
    impl<'de, T> serde::Deserialize<'de> for Node<T> {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            enum Field {
                Children,
                Parent,
            }
            impl<'de> serde::Deserialize<'de> for Field {
                fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    struct FieldVisitor;

                    impl<'de> serde::de::Visitor<'de> for FieldVisitor {
                        type Value = Field;

                        fn expecting(
                            &self,
                            formatter: &mut std::fmt::Formatter,
                        ) -> std::fmt::Result {
                            formatter.write_str("`children` or `parent`")
                        }

                        fn visit_str<E>(self, value: &str) -> Result<Field, E>
                        where
                            E: serde::de::Error,
                        {
                            match value {
                                "children" => Ok(Field::Children),
                                "parent" => Ok(Field::Parent),
                                _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                            }
                        }
                    }

                    deserializer.deserialize_identifier(FieldVisitor)
                }
            }

            struct IdVisitor<T> {
                phantom_data: std::marker::PhantomData<T>,
            }

            impl<'de, T> serde::de::Visitor<'de> for IdVisitor<T> {
                type Value = Node<T>;
                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, "struct Node")
                }
                fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::MapAccess<'de>,
                {
                    let mut children: Option<Vec<Id<T>>> = None;
                    let mut parent: Option<Option<Id<T>>> = None;
                    while let Some(key) = map.next_key()? {
                        match key {
                            Field::Children => {
                                if children.is_some() {
                                    return Err(serde::de::Error::duplicate_field("children"));
                                }
                                children = Some(map.next_value()?);
                            }
                            Field::Parent => {
                                if parent.is_some() {
                                    return Err(serde::de::Error::duplicate_field("parent"));
                                }
                                parent = Some(map.next_value()?);
                            }
                        }
                    }

                    Ok(Node {
                        children: children.ok_or(serde::de::Error::missing_field("children"))?,
                        parent: parent.ok_or(serde::de::Error::missing_field("parent"))?,
                    })
                }
            }

            const FIELDS: &[&str] = &["children", "parent"];
            deserializer.deserialize_struct(
                "Id",
                FIELDS,
                IdVisitor::<T> {
                    phantom_data: std::marker::PhantomData,
                },
            )
        }
    }
    impl<T> core::fmt::Debug for Node<T> {
        fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
            let Node { children, parent } = self;
            f.debug_struct("Node")
                .field("children", &children)
                .field("parent", &parent)
                .finish()
        }
    }

    impl<T> Default for IdTree<T> {
        fn default() -> Self {
            Self::new(Id::generate())
        }
    }
    impl<T> serde::Serialize for IdTree<T> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            use serde::ser::SerializeStruct;
            let mut serialized = serializer.serialize_struct("Node", 2)?;
            serialized.serialize_field("root_id", &self.root_id)?;
            serialized.serialize_field("nodes", &self.nodes)?;
            serialized.end()
        }
    }
    impl<'de, T> serde::Deserialize<'de> for IdTree<T> {
        fn deserialize<__D>(__deserializer: __D) -> serde::__private::Result<Self, __D::Error>
        where
            __D: serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __Field {
                __field0,
                __field1,
                __ignore,
            }
            #[doc(hidden)]
            struct __FieldVisitor;

            impl<'de> serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut serde::__private::Formatter,
                ) -> serde::__private::fmt::Result {
                    serde::__private::Formatter::write_str(__formatter, "field identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> serde::__private::Result<Self::Value, __E>
                where
                    __E: serde::de::Error,
                {
                    match __value {
                        0u64 => serde::__private::Ok(__Field::__field0),
                        1u64 => serde::__private::Ok(__Field::__field1),
                        _ => serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(self, __value: &str) -> serde::__private::Result<Self::Value, __E>
                where
                    __E: serde::de::Error,
                {
                    match __value {
                        "root_id" => serde::__private::Ok(__Field::__field0),
                        "nodes" => serde::__private::Ok(__Field::__field1),
                        _ => serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> serde::__private::Result<Self::Value, __E>
                where
                    __E: serde::de::Error,
                {
                    match __value {
                        b"root_id" => serde::__private::Ok(__Field::__field0),
                        b"nodes" => serde::__private::Ok(__Field::__field1),
                        _ => serde::__private::Ok(__Field::__ignore),
                    }
                }
            }
            impl<'de> serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> serde::__private::Result<Self, __D::Error>
                where
                    __D: serde::Deserializer<'de>,
                {
                    serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            #[doc(hidden)]
            struct __Visitor<'de, T> {
                marker: serde::__private::PhantomData<IdTree<T>>,
                lifetime: serde::__private::PhantomData<&'de ()>,
            }
            impl<'de, T> serde::de::Visitor<'de> for __Visitor<'de, T> {
                type Value = IdTree<T>;
                fn expecting(
                    &self,
                    __formatter: &mut serde::__private::Formatter,
                ) -> serde::__private::fmt::Result {
                    serde::__private::Formatter::write_str(__formatter, "struct IdTree")
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: serde::de::SeqAccess<'de>,
                {
                    let __field0 = match serde::de::SeqAccess::next_element::<Id<T>>(&mut __seq)? {
                        serde::__private::Some(__value) => __value,
                        serde::__private::None => {
                            return serde::__private::Err(serde::de::Error::invalid_length(
                                0usize,
                                &"struct IdTree with 2 elements",
                            ))
                        }
                    };
                    let __field1 = match serde::de::SeqAccess::next_element::<
                        BTreeMap<Id<T>, Node<T>>,
                    >(&mut __seq)?
                    {
                        serde::__private::Some(__value) => __value,
                        serde::__private::None => {
                            return serde::__private::Err(serde::de::Error::invalid_length(
                                1usize,
                                &"struct IdTree with 2 elements",
                            ))
                        }
                    };
                    serde::__private::Ok(IdTree {
                        root_id: __field0,
                        nodes: __field1,
                    })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: serde::de::MapAccess<'de>,
                {
                    let mut __field0: serde::__private::Option<Id<T>> = serde::__private::None;
                    let mut __field1: serde::__private::Option<BTreeMap<Id<T>, Node<T>>> =
                        serde::__private::None;
                    while let serde::__private::Some(__key) =
                        serde::de::MapAccess::next_key::<__Field>(&mut __map)?
                    {
                        match __key {
                            __Field::__field0 => {
                                if serde::__private::Option::is_some(&__field0) {
                                    return serde::__private::Err(
                                        <__A::Error as serde::de::Error>::duplicate_field(
                                            "root_id",
                                        ),
                                    );
                                }
                                __field0 =
                                    serde::__private::Some(serde::de::MapAccess::next_value::<
                                        Id<T>,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            __Field::__field1 => {
                                if serde::__private::Option::is_some(&__field1) {
                                    return serde::__private::Err(
                                        <__A::Error as serde::de::Error>::duplicate_field("nodes"),
                                    );
                                }
                                __field1 =
                                    serde::__private::Some(serde::de::MapAccess::next_value::<
                                        BTreeMap<Id<T>, Node<T>>,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            _ => {
                                let _ = serde::de::MapAccess::next_value::<serde::de::IgnoredAny>(
                                    &mut __map,
                                )?;
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        serde::__private::Some(__field0) => __field0,
                        serde::__private::None => serde::__private::de::missing_field("root_id")?,
                    };
                    let __field1 = match __field1 {
                        serde::__private::Some(__field1) => __field1,
                        serde::__private::None => serde::__private::de::missing_field("nodes")?,
                    };
                    serde::__private::Ok(IdTree {
                        root_id: __field0,
                        nodes: __field1,
                    })
                }
            }
            #[doc(hidden)]
            const FIELDS: &[&str] = &["root_id", "nodes"];
            serde::Deserializer::deserialize_struct(
                __deserializer,
                "IdTree",
                FIELDS,
                __Visitor {
                    marker: serde::__private::PhantomData::<IdTree<T>>,
                    lifetime: serde::__private::PhantomData,
                },
            )
        }
    }
    impl<T> core::clone::Clone for IdTree<T> {
        fn clone(&self) -> Self {
            let IdTree { root_id, nodes } = self;
            IdTree {
                root_id: *root_id,
                nodes: nodes.clone(),
            }
        }
    }
    impl<T> core::fmt::Debug for IdTree<T> {
        fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
            let IdTree { root_id, nodes } = self;
            f.debug_struct("IdTree")
                .field("root_id", &root_id)
                .field("nodes", &nodes)
                .finish()
        }
    }
}
