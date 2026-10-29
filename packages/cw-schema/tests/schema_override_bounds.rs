#![allow(dead_code, non_snake_case)]
// These syntax forms exercise value/type name collisions in schema expressions.
#![allow(clippy::let_and_return, clippy::single_element_loop)]

use cw_schema::{schema_of, NodeType, Schema, Schemaifier};

struct Unsupported;

fn root_type<T: Schemaifier>() -> NodeType {
    match schema_of::<T>() {
        Schema::V1(schema) => schema.definitions[schema.root].value.clone(),
        _ => unreachable!(),
    }
}

#[test]
fn block_override_exempts_skipped_generic() {
    #[derive(Schemaifier)]
    #[schemaifier(type = { cw_schema::NodeType::Unit })]
    struct Override<T> {
        #[serde(skip)]
        hidden: T,
    }

    assert_eq!(root_type::<Override<Unsupported>>(), NodeType::Unit);
}

#[test]
fn control_flow_override_exempts_unreferenced_generic() {
    #[derive(Schemaifier)]
    #[schemaifier(type = {
        let choose = |flag: bool| {
            if flag {
                cw_schema::NodeType::Unit
            } else {
                match Some(()) {
                    Some(()) => cw_schema::NodeType::Unit,
                    None => cw_schema::NodeType::Boolean,
                }
            }
        };
        let mut value = const { cw_schema::NodeType::Unit };
        for flag in [false, true] {
            value = choose(flag);
        }
        value
    })]
    struct Override<T> {
        #[serde(skip)]
        hidden: T,
    }

    #[derive(Schemaifier)]
    #[schemaifier(type = {
        let _ = std::mem::size_of::<std::string::String>();
        cw_schema::NodeType::Unit
    })]
    struct QualifiedOverride<String> {
        #[serde(skip)]
        hidden: String,
    }

    assert_eq!(root_type::<Override<Unsupported>>(), NodeType::Unit);
    assert_eq!(
        root_type::<QualifiedOverride<Unsupported>>(),
        NodeType::Unit
    );
}

#[test]
fn nested_override_preserves_bounds_for_referenced_generic() {
    #[derive(Schemaifier)]
    #[schemaifier(type = {
        let node = || root_type::<U>();
        node()
    })]
    struct Override<T, U> {
        #[serde(skip)]
        hidden: (T, U),
    }

    #[derive(Schemaifier)]
    #[schemaifier(type = {
        { let U = cw_schema::NodeType::Unit; let _ = U; }
        let T = root_type::<U>();
        T
    })]
    struct Scoped<T, U> {
        #[serde(skip)]
        hidden: (T, U),
    }

    #[derive(Schemaifier)]
    #[schemaifier(type = { let U = cw_schema::NodeType::Unit; let _ = U; root_type::<U>() })]
    struct ValueAndType<T, U> {
        #[serde(skip)]
        hidden: (T, U),
    }

    mod value_module {
        pub const U: bool = true;
    }

    #[derive(Schemaifier)]
    #[schemaifier(type = {
        use value_module::U;
        if U { root_type::<U>() } else { cw_schema::NodeType::Unit }
    })]
    struct ImportedValue<T, U> {
        #[serde(skip)]
        hidden: (T, U),
    }

    assert_eq!(
        root_type::<Override<Unsupported, bool>>(),
        NodeType::Boolean
    );
    assert_eq!(root_type::<Scoped<Unsupported, bool>>(), NodeType::Boolean);
    assert_eq!(
        root_type::<ValueAndType<Unsupported, bool>>(),
        NodeType::Boolean
    );
    assert_eq!(
        root_type::<ImportedValue<Unsupported, bool>>(),
        NodeType::Boolean
    );
}

#[test]
fn macro_override_exempts_unrelated_generic_names() {
    macro_rules! unit_schema {
        ($($ty:ty),+ $(,)?) => {
            cw_schema::NodeType::Unit
        };
    }
    macro_rules! labeled_unit {
        ($label:literal) => {
            cw_schema::NodeType::Unit
        };
    }

    #[derive(Schemaifier)]
    #[schemaifier(type = unit_schema!(std::string::String))]
    struct Qualified<String> {
        #[serde(skip)]
        hidden: String,
    }

    #[derive(Schemaifier)]
    #[schemaifier(type = unit_schema!(::std::string::String, Vec<std::string::String>,))]
    struct Nested<String> {
        #[serde(skip)]
        hidden: String,
    }

    #[derive(Schemaifier)]
    #[schemaifier(type = labeled_unit!("String"))]
    struct Literal<String> {
        #[serde(skip)]
        hidden: String,
    }

    assert_eq!(root_type::<Qualified<Unsupported>>(), NodeType::Unit);
    assert_eq!(root_type::<Nested<Unsupported>>(), NodeType::Unit);
    #[derive(Schemaifier)]
    #[schemaifier(type = unit_schema!(T))]
    struct Discarded<T> {
        #[serde(skip)]
        hidden: T,
    }

    assert_eq!(root_type::<Literal<Unsupported>>(), NodeType::Unit);
    assert_eq!(root_type::<Discarded<Unsupported>>(), NodeType::Unit);
}

#[test]
fn value_bindings_in_overrides_do_not_bound_namesake_generics() {
    #[derive(Schemaifier)]
    #[schemaifier(type = { let T = cw_schema::NodeType::Unit; T })]
    struct Local<T> {
        #[serde(skip)]
        hidden: T,
    }

    #[derive(Schemaifier)]
    #[schemaifier(type = {
        let choose = |T| T;
        let mut result = cw_schema::NodeType::Unit;
        for T in [cw_schema::NodeType::Boolean] {
            result = choose(T);
        }
        result
    })]
    struct ClosureAndLoop<T> {
        #[serde(skip)]
        hidden: T,
    }

    assert_eq!(root_type::<Local<Unsupported>>(), NodeType::Unit);
    assert_eq!(
        root_type::<ClosureAndLoop<Unsupported>>(),
        NodeType::Boolean
    );
}

#[test]
fn macro_override_preserves_referenced_generic_bounds() {
    macro_rules! schema_type {
        ($ty:ty) => {
            root_type::<$ty>()
        };
    }
    macro_rules! schema_expression {
        ($value:expr, $fallback:expr) => {
            $value.unwrap_or($fallback)
        };
    }

    #[derive(Schemaifier)]
    #[schemaifier(type = schema_type!(U))]
    struct TypeArgument<T, U: Schemaifier> {
        #[serde(skip)]
        hidden: (T, U),
    }

    #[derive(Schemaifier)]
    #[schemaifier(type = { let U = cw_schema::NodeType::Unit; let _ = U; schema_type!(U) })]
    struct ValueAndType<T, U: Schemaifier> {
        #[serde(skip)]
        hidden: (T, U),
    }

    #[derive(Schemaifier)]
    #[schemaifier(type = schema_expression!(Some(root_type::<U>()), cw_schema::NodeType::Unit))]
    struct ExpressionArgument<T, U>
    where
        U: Schemaifier,
    {
        #[serde(skip)]
        hidden: (T, U),
    }

    assert_eq!(
        root_type::<TypeArgument<Unsupported, bool>>(),
        NodeType::Boolean
    );
    assert_eq!(
        root_type::<ExpressionArgument<Unsupported, bool>>(),
        NodeType::Boolean
    );
    assert_eq!(
        root_type::<ValueAndType<Unsupported, bool>>(),
        NodeType::Boolean
    );
}

#[test]
fn macro_overrides_respect_local_value_scopes() {
    macro_rules! schema_value {
        ($value:expr) => {
            $value
        };
    }

    #[derive(Schemaifier)]
    #[schemaifier(type = { let T = cw_schema::NodeType::Unit; schema_value!(T) })]
    struct Local<T> {
        #[serde(skip)]
        hidden: T,
    }

    #[derive(Schemaifier)]
    #[schemaifier(type = { let echo = |T| schema_value!(T); echo(cw_schema::NodeType::Unit) })]
    struct Closure<T> {
        #[serde(skip)]
        hidden: T,
    }

    #[derive(Schemaifier)]
    #[schemaifier(type = {
        let mut node = cw_schema::NodeType::Boolean;
        for T in [cw_schema::NodeType::Unit] { node = schema_value!(T); }
        node
    })]
    struct ForLoop<T> {
        #[serde(skip)]
        hidden: T,
    }

    #[derive(Schemaifier)]
    #[schemaifier(type = {
        match Some(cw_schema::NodeType::Unit) {
            Some(T) => schema_value!(T),
            None => cw_schema::NodeType::Boolean,
        }
    })]
    struct Match<T> {
        #[serde(skip)]
        hidden: T,
    }

    #[derive(Schemaifier)]
    #[schemaifier(type = {
        if let Some(T) = Some(cw_schema::NodeType::Unit) {
            schema_value!(T)
        } else { cw_schema::NodeType::Boolean }
    })]
    struct IfLet<T> {
        #[serde(skip)]
        hidden: T,
    }

    #[derive(Schemaifier)]
    #[schemaifier(type = {
        let mut next = Some(cw_schema::NodeType::Unit);
        let mut node = cw_schema::NodeType::Boolean;
        while let Some(T) = next.take() { node = schema_value!(T); }
        node
    })]
    struct WhileLet<T> {
        #[serde(skip)]
        hidden: T,
    }

    assert_eq!(root_type::<Local<Unsupported>>(), NodeType::Unit);
    assert_eq!(root_type::<Closure<Unsupported>>(), NodeType::Unit);
    assert_eq!(root_type::<ForLoop<Unsupported>>(), NodeType::Unit);
    assert_eq!(root_type::<Match<Unsupported>>(), NodeType::Unit);
    assert_eq!(root_type::<IfLet<Unsupported>>(), NodeType::Unit);
    assert_eq!(root_type::<WhileLet<Unsupported>>(), NodeType::Unit);
}

#[test]
fn mixed_macro_arguments_preserve_generic_type_bounds() {
    macro_rules! schema_type {
        ([$ty:ty => $enabled:expr]) => {
            schema_type!($ty, $enabled)
        };
        (@schema $ty:ty => $enabled:expr) => {
            schema_type!($ty, $enabled)
        };
        ($ty:ty, $enabled:expr) => {
            if $enabled {
                root_type::<$ty>()
            } else {
                cw_schema::NodeType::Unit
            }
        };
        ($ty:ty; $enabled:expr) => {
            schema_type!($ty, $enabled)
        };
    }
    macro_rules! schema_value {
        ($value:expr) => {
            $value
        };
        ($value:expr; $enabled:expr) => {
            if $enabled {
                $value
            } else {
                cw_schema::NodeType::Unit
            }
        };
    }

    #[derive(Schemaifier)]
    #[schemaifier(type = schema_type!(U, true))]
    struct Mixed<T, U: Schemaifier> {
        #[serde(skip)]
        hidden: (T, U),
    }

    #[derive(Schemaifier)]
    #[schemaifier(type = schema_type!(Option<U>, true))]
    struct Nested<T, U: Schemaifier> {
        #[serde(skip)]
        hidden: (T, U),
    }

    #[derive(Schemaifier)]
    #[schemaifier(type = {
        { let U = cw_schema::NodeType::Unit; let _ = schema_value!(U); }
        let T = schema_type!(U, true);
        schema_value!(T)
    })]
    struct Scoped<T, U: Schemaifier> {
        #[serde(skip)]
        hidden: (T, U),
    }

    #[derive(Schemaifier)]
    #[schemaifier(type = schema_type!(U; true))]
    struct Semicolon<T, U: Schemaifier> {
        #[serde(skip)]
        hidden: (T, U),
    }

    #[derive(Schemaifier)]
    #[schemaifier(type = schema_type!([Option<U> => true]))]
    struct Grouped<T, U: Schemaifier> {
        #[serde(skip)]
        hidden: (T, U),
    }

    #[derive(Schemaifier)]
    #[schemaifier(type = schema_type!(@schema U => true))]
    struct Custom<T, U: Schemaifier> {
        #[serde(skip)]
        hidden: (T, U),
    }

    #[derive(Schemaifier)]
    #[schemaifier(type = schema_type!([std::string::String => true]))]
    struct Qualified<String> {
        #[serde(skip)]
        hidden: String,
    }

    #[derive(Schemaifier)]
    #[schemaifier(type = {
        let T = schema_type!(U; true);
        schema_value!(T; true)
    })]
    struct ScopedSeparators<T, U: Schemaifier> {
        #[serde(skip)]
        hidden: (T, U),
    }

    assert_eq!(root_type::<Mixed<Unsupported, bool>>(), NodeType::Boolean);
    assert_eq!(
        root_type::<Nested<Unsupported, bool>>(),
        root_type::<Option<bool>>()
    );
    assert_eq!(root_type::<Scoped<Unsupported, bool>>(), NodeType::Boolean);
    assert_eq!(
        root_type::<Semicolon<Unsupported, bool>>(),
        NodeType::Boolean
    );
    assert_eq!(
        root_type::<Grouped<Unsupported, bool>>(),
        root_type::<Option<bool>>()
    );
    assert_eq!(root_type::<Custom<Unsupported, bool>>(), NodeType::Boolean);
    assert_eq!(
        root_type::<Qualified<Unsupported>>(),
        root_type::<std::string::String>()
    );
    assert_eq!(
        root_type::<ScopedSeparators<Unsupported, bool>>(),
        NodeType::Boolean
    );
}
