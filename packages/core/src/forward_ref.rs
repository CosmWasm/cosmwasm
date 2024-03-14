/// # ⚠ THIS IS AN INTERNAL IMPLEMENTATION DETAIL. DO NOT USE.
///
/// Given an implementation of `T == U`, implements:
/// - `&T == U`
/// - `T == &U`
///
/// We don't need to add `&T == &U` here because this is implemented automatically.
#[doc(hidden)]
#[macro_export]
macro_rules! __internal__forward_ref_partial_eq {
    ($t:ty, $u:ty) => {
        // `&T == U`
        impl<'a> PartialEq<$u> for &'a $t {
            #[inline]
            fn eq(&self, rhs: &$u) -> bool {
                **self == *rhs // Implement via T == U
            }
        }

        // `T == &U`
        impl PartialEq<&$u> for $t {
            #[inline]
            fn eq(&self, rhs: &&$u) -> bool {
                *self == **rhs // Implement via T == U
            }
        }
    };
}

/// implements binary operators "&T op U", "T op &U", "&T op &U"
/// based on "T op U" where T and U are expected to be `Copy`able
///
/// Copied from `libcore`
macro_rules! forward_ref_binop {
    (impl $imp:ident, $method:ident for $t:ty, $u:ty) => {
        impl<'a> $imp<$u> for &'a $t {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            #[track_caller]
            fn $method(self, other: $u) -> <$t as $imp<$u>>::Output {
                $imp::$method(*self, other)
            }
        }

        impl $imp<&$u> for $t {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            #[track_caller]
            fn $method(self, other: &$u) -> <$t as $imp<$u>>::Output {
                $imp::$method(self, *other)
            }
        }

        impl $imp<&$u> for &$t {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            #[track_caller]
            fn $method(self, other: &$u) -> <$t as $imp<$u>>::Output {
                $imp::$method(*self, *other)
            }
        }
    };
}

/// implements "T op= &U", based on "T op= U"
/// where U is expected to be `Copy`able
///
/// Copied from `libcore`
macro_rules! forward_ref_op_assign {
    (impl $imp:ident, $method:ident for $t:ty, $u:ty) => {
        impl $imp<&$u> for $t {
            #[inline]
            #[track_caller]
            fn $method(&mut self, other: &$u) {
                $imp::$method(self, *other);
            }
        }
    };
}

pub(crate) use {forward_ref_binop, forward_ref_op_assign};
