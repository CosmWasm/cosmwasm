/// Implements a hidden constructor for non-exhaustive structures that need
/// to be built in libraries like cw-multi-test.
macro_rules! impl_hidden_constructor {
    ( $response:ty, $( $field: ident : $t: ty),* ) => {
        impl $response {
            /// Constructor for testing frameworks such as cw-multi-test.
            /// This is required because the type is #[non_exhaustive].
            /// As a contract developer you should not need this constructor since
            /// the given structure is constructed for you via deserialization.
            ///
            /// Warning: This can change in breaking ways in minor versions.
            #[doc(hidden)]
            #[allow(dead_code)]
            pub fn new($( $field: $t),*) -> Self {
                Self { $( $field ),* }
            }
        }
    };
}

pub(crate) use impl_hidden_constructor;
