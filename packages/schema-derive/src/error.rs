macro_rules! bail {
    ($span_src:expr, $msg:literal) => {{
        return Err($crate::error::error_message!($span_src, $msg));
    }};
}

macro_rules! error_message {
    ($span_src:expr, $msg:literal) => {{
        ::syn::Error::new(::syn::spanned::Spanned::span(&{ $span_src }), $msg)
    }};
}

macro_rules! fallible_macro {
    (
        $(
            #[ $( $attribute_decl:tt )* ]
        )*
        pub fn $macro_name:ident ( $( $params:tt )* ) -> syn::Result<$inner_return:path> {
            $( $fn_body:tt )*
        }
    ) => {
        $(
            #[ $( $attribute_decl )* ]
        )*
        pub fn $macro_name ( $( $params )* ) -> $inner_return {
            let result = move || -> syn::Result<_> {
                $( $fn_body )*
            };

            match result() {
                Ok(val) => val,
                Err(err) => err.into_compile_error().into(),
            }
        }
    }
}

pub(crate) use bail;
pub(crate) use error_message;
pub(crate) use fallible_macro;
