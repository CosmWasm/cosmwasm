use crate::{Node, NodeType, Schemaifier};
use alloc::{borrow::Cow, string::String, vec, vec::Vec};

impl Schemaifier for String {
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

impl_integer!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128);

impl Schemaifier for f32 {
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
