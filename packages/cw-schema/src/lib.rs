#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use alloc::{borrow::Cow, collections::BTreeMap, vec::Vec};
use core::{any::TypeId, hash::BuildHasherDefault, marker::PhantomData};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use siphasher::sip::SipHasher;

pub use cw_schema_derive::Schemaifier;

pub type DefinitionReference = usize;

mod default_impls;

#[skip_serializing_none]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "std", derive(::schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct StructProperty {
    pub description: Option<Cow<'static, str>>,
    pub value: DefinitionReference,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "std", derive(::schemars::JsonSchema))]
#[serde(rename_all = "camelCase", untagged)]
pub enum StructType {
    Unit,
    Named {
        properties: BTreeMap<Cow<'static, str>, StructProperty>,
    },
    Tuple {
        items: Vec<DefinitionReference>,
    },
}

#[skip_serializing_none]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "std", derive(::schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct EnumCase {
    pub description: Option<Cow<'static, str>>,
    #[serde(flatten)]
    pub value: EnumValue,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "std", derive(::schemars::JsonSchema))]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum EnumValue {
    Unit,
    Named {
        properties: BTreeMap<Cow<'static, str>, StructProperty>,
    },
    Tuple {
        items: Vec<DefinitionReference>,
    },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "std", derive(::schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum MapKind {
    BTree,
    Hash,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "std", derive(::schemars::JsonSchema))]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum NodeType {
    // Floating point numbers
    Float,
    Double,

    // Decimal numbers
    Decimal {
        precision: u64,
        signed: bool,
    },

    // Integer numbers
    Integer {
        precision: u64,
        signed: bool,
    },

    Address,
    Binary,
    Checksum,
    HexBinary,
    Timestamp,

    String,
    Boolean,
    Array {
        items: DefinitionReference,
    },
    Struct(StructType),
    Tuple {
        items: Vec<DefinitionReference>,
    },
    Enum {
        discriminator: Option<Cow<'static, str>>,
        cases: BTreeMap<Cow<'static, str>, EnumCase>,
    },

    Map {
        kind: MapKind,
        key: DefinitionReference,
        value: DefinitionReference,
    },

    Optional {
        inner: DefinitionReference,
    },
    Unit,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "std", derive(::schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub name: Cow<'static, str>,
    pub description: Option<Cow<'static, str>>,
    #[serde(flatten)]
    pub value: NodeType,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "std", derive(::schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SchemaV1 {
    pub root: DefinitionReference,
    pub definitions: Vec<Node>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "std", derive(::schemars::JsonSchema))]
#[serde(rename_all = "camelCase", tag = "type")]
#[non_exhaustive]
pub enum Schema {
    V1(SchemaV1),
}

#[derive(Hash, PartialEq, Eq)]
pub struct Identifier(TypeId);

impl Identifier {
    pub fn of<T>() -> Self
    where
        T: ?Sized,
    {
        // Taken from <sagebind/castaway>: https://github.com/sagebind/castaway/blob/a7baeab32d75d0f105d1415210a2867d213f8818/src/utils.rs#L36
        //
        // Seems more robust than the previous implementation.
        trait NonStaticAny {
            fn get_type_id(&self) -> TypeId
            where
                Self: 'static;
        }

        impl<T: ?Sized> NonStaticAny for PhantomData<T> {
            fn get_type_id(&self) -> TypeId
            where
                Self: 'static,
            {
                TypeId::of::<T>()
            }
        }

        let phantom = PhantomData::<T>;
        let ty_id = NonStaticAny::get_type_id(unsafe {
            core::mem::transmute::<&dyn NonStaticAny, &(dyn NonStaticAny + 'static)>(&phantom)
        });

        Identifier(ty_id)
    }
}

#[derive(Default)]
pub struct SchemaVisitor {
    schemas: IndexMap<Identifier, Node, BuildHasherDefault<SipHasher>>,
}

impl SchemaVisitor {
    pub fn get_reference<T: Schemaifier>(&self) -> Option<DefinitionReference> {
        self.schemas.get_index_of(&T::id())
    }

    pub fn get_schema<T: Schemaifier>(&self) -> Option<&Node> {
        self.schemas.get(&T::id())
    }

    pub fn insert(&mut self, id: Identifier, node: Node) -> DefinitionReference {
        let (id, _) = self.schemas.insert_full(id, node);
        id
    }

    /// Transform this visitor into a vector where the `DefinitionReference` can be used as an index
    /// to access the schema of the particular node.
    pub fn into_vec(self) -> Vec<Node> {
        self.schemas.into_values().collect()
    }
}

pub trait Schemaifier {
    fn id() -> Identifier {
        Identifier::of::<Self>()
    }

    fn visit_schema(visitor: &mut SchemaVisitor) -> DefinitionReference;
}

pub fn schema_of<T: Schemaifier + ?Sized>() -> Schema {
    let mut visitor = SchemaVisitor::default();
    Schema::V1(SchemaV1 {
        root: T::visit_schema(&mut visitor),
        definitions: visitor.into_vec(),
    })
}

#[doc(hidden)]
pub mod reexport {
    pub use alloc::collections::BTreeMap;
}
