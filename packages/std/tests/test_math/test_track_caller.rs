//! Checks that panicking math operations report the location of the caller
//! (this file) instead of a location inside cosmwasm-std.

use cosmwasm_std::{
    Decimal, Decimal256, Int128, Int256, Int512, Int64, SignedDecimal, SignedDecimal256, Uint128,
    Uint256, Uint512, Uint64,
};
use std::cell::RefCell;
use std::panic::{self, UnwindSafe};
use std::sync::Once;

thread_local! {
    static LAST_PANIC_LOCATION: RefCell<Option<(String, u32)>> = const { RefCell::new(None) };
}

static INSTALL_HOOK: Once = Once::new();

/// Runs `f`, which must panic, and returns the file and line the panic was reported at.
fn panic_location(f: impl FnOnce() + UnwindSafe) -> (String, u32) {
    INSTALL_HOOK.call_once(|| {
        let previous = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            if let Some(location) = info.location() {
                LAST_PANIC_LOCATION.with(|last| {
                    *last.borrow_mut() = Some((location.file().to_string(), location.line()));
                });
            }
            previous(info);
        }));
    });
    LAST_PANIC_LOCATION.with(|last| last.borrow_mut().take());
    assert!(panic::catch_unwind(f).is_err(), "expected a panic");
    LAST_PANIC_LOCATION
        .with(|last| last.borrow_mut().take())
        .expect("panic location was not captured")
}

/// Asserts that evaluating `$e` panics and that the panic is reported at the line
/// of the macro invocation in this file.
macro_rules! assert_panics_at_caller {
    ($e:expr) => {{
        let (file, line) = panic_location(|| {
            let _ = $e;
        });
        assert_eq!(file, file!());
        assert_eq!(line, line!());
    }};
}

#[test]
#[allow(clippy::op_ref)] // by-reference operators are tested on purpose
fn uint_panics_report_caller_location() {
    assert_panics_at_caller!(Uint64::MAX + Uint64::one());
    assert_panics_at_caller!(Uint128::zero() - Uint128::one());
    assert_panics_at_caller!(Uint256::MAX * Uint256::from(2u32));
    assert_panics_at_caller!(Uint512::one() / Uint512::zero());
    assert_panics_at_caller!(Uint128::one() % Uint128::zero());
    assert_panics_at_caller!(Uint256::one() >> 256u32);
    assert_panics_at_caller!(Uint128::one() << 128u32);
    assert_panics_at_caller!(Uint128::MAX.pow(2));
    assert_panics_at_caller!(Uint128::MAX.strict_add(Uint128::one()));
    assert_panics_at_caller!(Uint128::one().multiply_ratio(1u128, 0u128));
    assert_panics_at_caller!(Uint128::MAX.mul_floor((2u128, 1u128)));
    assert_panics_at_caller!(&Uint128::MAX + &Uint128::one());
}

#[test]
fn op_assign_panics_report_caller_location() {
    let mut a = Uint128::zero();
    let expected_line = line!() + 1;
    let (file, line) = panic_location(move || a -= Uint128::one());
    assert_eq!(file, file!());
    assert_eq!(line, expected_line);

    let mut b = Decimal::MAX;
    let expected_line = line!() + 1;
    let (file, line) = panic_location(move || b += Decimal::one());
    assert_eq!(file, file!());
    assert_eq!(line, expected_line);
}

#[test]
fn int_panics_report_caller_location() {
    assert_panics_at_caller!(Int64::MAX + Int64::one());
    assert_panics_at_caller!(Int128::MIN - Int128::one());
    assert_panics_at_caller!(Int256::MAX * Int256::from(2));
    assert_panics_at_caller!(Int512::one() / Int512::zero());
    assert_panics_at_caller!(-Int128::MIN);
    assert_panics_at_caller!(Int128::MIN.abs());
    assert_panics_at_caller!(Int256::one() >> 256u32);
}

#[test]
fn decimal_panics_report_caller_location() {
    assert_panics_at_caller!(Decimal::MAX + Decimal::one());
    assert_panics_at_caller!(Decimal256::zero() - Decimal256::one());
    assert_panics_at_caller!(Decimal::MAX * Decimal::percent(200));
    assert_panics_at_caller!(Decimal::one() / Decimal::zero());
    assert_panics_at_caller!(Decimal::from_ratio(1u128, 0u128));
    assert_panics_at_caller!(SignedDecimal::MIN - SignedDecimal::one());
    assert_panics_at_caller!(-SignedDecimal256::MIN);
    assert_panics_at_caller!(SignedDecimal::MAX.pow(2));
}
