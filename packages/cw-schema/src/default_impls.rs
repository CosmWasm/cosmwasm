use crate::{MapKind, Node, NodeType, Schemaifier};
use alloc::{
    borrow::{Cow, ToOwned},
    collections::BTreeMap,
    string::String,
    vec,
    vec::Vec,
};

impl Schemaifier for () {
    #[inline]
    fn visit_schema(visitor: &mut crate::SchemaVisitor) -> crate::DefinitionReference {
        visitor.insert(
            Self::id(),
            Node {
                name: Cow::Borrowed("Unit"),
                description: None,
                value: NodeType::Unit,
            },
        )
    }
}

impl Schemaifier for str {
    #[inline]
    fn visit_schema(visitor: &mut crate::SchemaVisitor) -> crate::DefinitionReference {
        visitor.insert(
            Self::id(),
            Node {
                name: Cow::Borrowed("str"),
                description: None,
                value: NodeType::String,
            },
        )
    }
}

impl Schemaifier for String {
    #[inline]
    fn visit_schema(visitor: &mut crate::SchemaVisitor) -> crate::DefinitionReference {
        visitor.insert(
            Self::id(),
            Node {
                name: Cow::Borrowed("String"),
                description: None,
                value: NodeType::String,
            },
        )
    }
}

macro_rules! impl_integer {
    ($($t:ty),+) => {
        $(
            impl Schemaifier for $t {
                #[inline]
                fn visit_schema(visitor: &mut crate::SchemaVisitor) -> crate::DefinitionReference {
                    visitor.insert(Self::id(), Node {
                        name: Cow::Borrowed(stringify!($t)),
                        description: None,
                        value: NodeType::Integer {
                            signed: <$t>::MIN != 0,
                            precision: <$t>::BITS as u64,
                        },
                    })
                }
            }
        )+
    };
}

impl_integer!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);

impl Schemaifier for f32 {
    #[inline]
    fn visit_schema(visitor: &mut crate::SchemaVisitor) -> crate::DefinitionReference {
        visitor.insert(
            Self::id(),
            Node {
                name: Cow::Borrowed("f32"),
                description: None,
                value: NodeType::Float,
            },
        )
    }
}

impl Schemaifier for f64 {
    #[inline]
    fn visit_schema(visitor: &mut crate::SchemaVisitor) -> crate::DefinitionReference {
        visitor.insert(
            Self::id(),
            Node {
                name: Cow::Borrowed("f64"),
                description: None,
                value: NodeType::Double,
            },
        )
    }
}

impl Schemaifier for bool {
    #[inline]
    fn visit_schema(visitor: &mut crate::SchemaVisitor) -> crate::DefinitionReference {
        visitor.insert(
            Self::id(),
            Node {
                name: Cow::Borrowed("bool"),
                description: None,
                value: NodeType::Boolean,
            },
        )
    }
}

impl<T> Schemaifier for Vec<T>
where
    T: Schemaifier,
{
    #[inline]
    fn visit_schema(visitor: &mut crate::SchemaVisitor) -> crate::DefinitionReference {
        let node = Node {
            name: Cow::Borrowed(std::any::type_name::<Self>()),
            description: None,
            value: NodeType::Array {
                items: T::visit_schema(visitor),
            },
        };

        visitor.insert(Self::id(), node)
    }
}

macro_rules! all_the_tuples {
    ($($($n:ident),+);+$(;)?) => {
        $(
            impl<$($n: Schemaifier),+> Schemaifier for ($($n,)+) {
                #[inline]
                fn visit_schema(visitor: &mut crate::SchemaVisitor) -> crate::DefinitionReference {
                    let node = Node {
                        name: Cow::Borrowed(std::any::type_name::<Self>()),
                        description: None,
                        value: NodeType::Tuple {
                            items: vec![
                                $(<$n as Schemaifier>::visit_schema(visitor)),+
                            ],
                        },
                    };

                    visitor.insert(Self::id(), node)
                }
            }
        )+
    };
}

// Implement for tuples up to 16 elements.
// Good enough. If someone needs more, PR it.
all_the_tuples! {
    A;
    A, B;
    A, B, C;
    A, B, C, D;
    A, B, C, D, E;
    A, B, C, D, E, F;
    A, B, C, D, E, F, G;
    A, B, C, D, E, F, G, H;
    A, B, C, D, E, F, G, H, I;
    A, B, C, D, E, F, G, H, I, J;
    A, B, C, D, E, F, G, H, I, J, K;
    A, B, C, D, E, F, G, H, I, J, K, L;
    A, B, C, D, E, F, G, H, I, J, K, L, M;
    A, B, C, D, E, F, G, H, I, J, K, L, M, N;
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O;
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P;
}

impl<T> Schemaifier for Option<T>
where
    T: Schemaifier,
{
    #[inline]
    fn visit_schema(visitor: &mut crate::SchemaVisitor) -> crate::DefinitionReference {
        let node = Node {
            name: Cow::Borrowed(std::any::type_name::<Self>()),
            description: None,
            value: NodeType::Optional {
                inner: T::visit_schema(visitor),
            },
        };

        visitor.insert(Self::id(), node)
    }
}

impl<K, V> Schemaifier for BTreeMap<K, V>
where
    K: Schemaifier,
    V: Schemaifier,
{
    #[inline]
    fn visit_schema(visitor: &mut crate::SchemaVisitor) -> crate::DefinitionReference {
        let node = Node {
            name: Cow::Borrowed(std::any::type_name::<Self>()),
            description: None,
            value: NodeType::Map {
                kind: MapKind::BTree,
                key: K::visit_schema(visitor),
                value: V::visit_schema(visitor),
            },
        };

        visitor.insert(Self::id(), node)
    }
}

#[cfg(feature = "std")]
impl<K, V, S> Schemaifier for std::collections::HashMap<K, V, S>
where
    K: Schemaifier,
    V: Schemaifier,
{
    #[inline]
    fn visit_schema(visitor: &mut crate::SchemaVisitor) -> crate::DefinitionReference {
        let node = Node {
            name: Cow::Borrowed(std::any::type_name::<Self>()),
            description: None,
            value: NodeType::Map {
                kind: MapKind::Hash,
                key: K::visit_schema(visitor),
                value: V::visit_schema(visitor),
            },
        };

        visitor.insert(Self::id(), node)
    }
}

impl<T> Schemaifier for &T
where
    T: Schemaifier + ?Sized,
{
    #[inline]
    fn visit_schema(visitor: &mut crate::SchemaVisitor) -> crate::DefinitionReference {
        T::visit_schema(visitor)
    }
}

impl<T> Schemaifier for Cow<'_, T>
where
    T: Schemaifier + ToOwned + ?Sized,
{
    #[inline]
    fn visit_schema(visitor: &mut crate::SchemaVisitor) -> crate::DefinitionReference {
        T::visit_schema(visitor)
    }
}
