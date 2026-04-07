use cosmwasm_std::{
    CheckedFromRatioError, Decimal, Decimal256, DecimalRangeExceeded, DivideByZeroError, Fraction,
    Int128, Int256, OverflowError, OverflowOperation, RoundUpOverflowError, SignedDecimal,
    SignedDecimal256, Uint128, Uint256,
};
use std::fmt::Write;
use std::str::FromStr;

/// 1*10^18
const DECIMAL_FRACTIONAL: Uint128 = Uint128::new(1_000_000_000_000_000_000);

fn dec(input: &str) -> Decimal {
    Decimal::from_str(input).unwrap()
}

#[test]
fn decimal_new() {
    let expected = Uint128::from(300u128);
    assert_eq!("0.0000000000000003", Decimal::new(expected).to_string());
}

#[test]
#[allow(deprecated)]
fn decimal_raw() {
    let value = 300u128;
    assert_eq!(Decimal::raw(value), Decimal::new(Uint128::from(300u128)));
}

#[test]
fn decimal_one() {
    let value = Decimal::one();
    assert_eq!(value, Decimal::new(DECIMAL_FRACTIONAL));
}

#[test]
fn decimal_zero() {
    let value = Decimal::zero();
    assert!(value.is_zero());
}

#[test]
fn decimal_percent() {
    let value = Decimal::percent(50);
    assert_eq!(value, Decimal::new(DECIMAL_FRACTIONAL / Uint128::from(2u8)));
}

#[test]
fn decimal_permille() {
    let value = Decimal::permille(125);
    assert_eq!(value, Decimal::new(DECIMAL_FRACTIONAL / Uint128::from(8u8)));
}

#[test]
fn decimal_bps() {
    let value = Decimal::bps(125);
    assert_eq!(
        value,
        Decimal::new(DECIMAL_FRACTIONAL / Uint128::from(80u8))
    );
}

#[test]
fn decimal_from_decimal256_works() {
    let too_big = Decimal256::new(Uint256::from(Uint128::MAX) + Uint256::one());
    assert_eq!(Decimal::try_from(too_big), Err(DecimalRangeExceeded));

    let just_right = Decimal256::new(Uint256::from(Uint128::MAX));
    assert_eq!(Decimal::try_from(just_right), Ok(Decimal::MAX));

    assert_eq!(Decimal::try_from(Decimal256::zero()), Ok(Decimal::zero()));
    assert_eq!(Decimal::try_from(Decimal256::one()), Ok(Decimal::one()));
    assert_eq!(
        Decimal::try_from(Decimal256::percent(50)),
        Ok(Decimal::percent(50))
    );
}

#[test]
fn decimal_from_signed_decimal256_works() {
    let too_big = SignedDecimal256::new(Int256::from(Int128::MAX) * Int256::from(Int128::MAX));
    assert_eq!(Decimal::try_from(too_big), Err(DecimalRangeExceeded));

    let just_right =
        SignedDecimal256::new(Int256::new(i128::MAX) + Int256::new(i128::MAX) + Int256::one());
    assert_eq!(Decimal::try_from(just_right), Ok(Decimal::MAX));

    assert_eq!(
        Decimal::try_from(SignedDecimal256::zero()),
        Ok(Decimal::zero())
    );
    assert_eq!(
        Decimal::try_from(SignedDecimal256::one()),
        Ok(Decimal::one())
    );
    assert_eq!(
        Decimal::try_from(SignedDecimal256::percent(50)),
        Ok(Decimal::percent(50))
    );
}

#[test]
fn decimal_try_from_integer() {
    let int = Uint128::new(0xDEADBEEF);
    let decimal = Decimal::try_from(int).unwrap();
    assert_eq!(int.to_string(), decimal.to_string());
}

#[test]
fn decimal_try_from_signed_works() {
    assert_eq!(
        Decimal::try_from(SignedDecimal::MAX).unwrap(),
        Decimal::new(Uint128::new(SignedDecimal::MAX.atomics().i128() as u128))
    );
    assert_eq!(
        Decimal::try_from(SignedDecimal::zero()).unwrap(),
        Decimal::zero()
    );
    assert_eq!(
        Decimal::try_from(SignedDecimal::one()).unwrap(),
        Decimal::one()
    );
    assert_eq!(
        Decimal::try_from(SignedDecimal::percent(50)).unwrap(),
        Decimal::percent(50)
    );
    assert_eq!(
        Decimal::try_from(SignedDecimal::negative_one()),
        Err(DecimalRangeExceeded)
    );
    assert_eq!(
        Decimal::try_from(SignedDecimal::MIN),
        Err(DecimalRangeExceeded)
    );
}

#[test]
fn decimal_from_atomics_works() {
    let one = Decimal::one();
    let two = one + one;

    assert_eq!(Decimal::from_atomics(1u128, 0).unwrap(), one);
    assert_eq!(Decimal::from_atomics(10u128, 1).unwrap(), one);
    assert_eq!(Decimal::from_atomics(100u128, 2).unwrap(), one);
    assert_eq!(Decimal::from_atomics(1000u128, 3).unwrap(), one);
    assert_eq!(
        Decimal::from_atomics(1000000000000000000u128, 18).unwrap(),
        one
    );
    assert_eq!(
        Decimal::from_atomics(10000000000000000000u128, 19).unwrap(),
        one
    );
    assert_eq!(
        Decimal::from_atomics(100000000000000000000u128, 20).unwrap(),
        one
    );

    assert_eq!(Decimal::from_atomics(2u128, 0).unwrap(), two);
    assert_eq!(Decimal::from_atomics(20u128, 1).unwrap(), two);
    assert_eq!(Decimal::from_atomics(200u128, 2).unwrap(), two);
    assert_eq!(Decimal::from_atomics(2000u128, 3).unwrap(), two);
    assert_eq!(
        Decimal::from_atomics(2000000000000000000u128, 18).unwrap(),
        two
    );
    assert_eq!(
        Decimal::from_atomics(20000000000000000000u128, 19).unwrap(),
        two
    );
    assert_eq!(
        Decimal::from_atomics(200000000000000000000u128, 20).unwrap(),
        two
    );

    // Cuts decimal digits (20 provided but only 18 can be stored)
    assert_eq!(
        Decimal::from_atomics(4321u128, 20).unwrap(),
        Decimal::from_str("0.000000000000000043").unwrap()
    );
    assert_eq!(
        Decimal::from_atomics(6789u128, 20).unwrap(),
        Decimal::from_str("0.000000000000000067").unwrap()
    );
    assert_eq!(
        Decimal::from_atomics(u128::MAX, 38).unwrap(),
        Decimal::from_str("3.402823669209384634").unwrap()
    );
    assert_eq!(
        Decimal::from_atomics(u128::MAX, 39).unwrap(),
        Decimal::from_str("0.340282366920938463").unwrap()
    );
    assert_eq!(
        Decimal::from_atomics(u128::MAX, 45).unwrap(),
        Decimal::from_str("0.000000340282366920").unwrap()
    );
    assert_eq!(
        Decimal::from_atomics(u128::MAX, 51).unwrap(),
        Decimal::from_str("0.000000000000340282").unwrap()
    );
    assert_eq!(
        Decimal::from_atomics(u128::MAX, 56).unwrap(),
        Decimal::from_str("0.000000000000000003").unwrap()
    );
    assert_eq!(
        Decimal::from_atomics(u128::MAX, 57).unwrap(),
        Decimal::from_str("0.000000000000000000").unwrap()
    );
    assert_eq!(
        Decimal::from_atomics(u128::MAX, u32::MAX).unwrap(),
        Decimal::from_str("0.000000000000000000").unwrap()
    );
    assert_eq!(
        Decimal::from_atomics(0u128, u32::MAX).unwrap(),
        Decimal::zero()
    );

    // Can be used with max value
    let max = Decimal::MAX;
    assert_eq!(
        Decimal::from_atomics(max.atomics(), max.decimal_places()).unwrap(),
        max
    );

    // Overflow is only possible with digits < 18
    let result = Decimal::from_atomics(u128::MAX, 17);
    assert_eq!(result.unwrap_err(), DecimalRangeExceeded);
}

#[test]
fn decimal_from_ratio_works() {
    // 1.0
    assert_eq!(Decimal::from_ratio(1u128, 1u128), Decimal::one());
    assert_eq!(Decimal::from_ratio(53u128, 53u128), Decimal::one());
    assert_eq!(Decimal::from_ratio(125u128, 125u128), Decimal::one());

    // 1.5
    assert_eq!(Decimal::from_ratio(3u128, 2u128), Decimal::percent(150));
    assert_eq!(Decimal::from_ratio(150u128, 100u128), Decimal::percent(150));
    assert_eq!(Decimal::from_ratio(333u128, 222u128), Decimal::percent(150));

    // 0.125
    assert_eq!(Decimal::from_ratio(1u64, 8u64), Decimal::permille(125));
    assert_eq!(Decimal::from_ratio(125u64, 1000u64), Decimal::permille(125));

    // 1/3 (result floored)
    assert_eq!(
        Decimal::from_ratio(1u64, 3u64),
        Decimal::new(Uint128::from(333_333_333_333_333_333u128))
    );

    // 2/3 (result floored)
    assert_eq!(
        Decimal::from_ratio(2u64, 3u64),
        Decimal::new(Uint128::from(666_666_666_666_666_666u128))
    );

    // large inputs
    assert_eq!(Decimal::from_ratio(0u128, u128::MAX), Decimal::zero());
    assert_eq!(Decimal::from_ratio(u128::MAX, u128::MAX), Decimal::one());
    // 340282366920938463463 is the largest integer <= Decimal::MAX
    assert_eq!(
        Decimal::from_ratio(340282366920938463463u128, 1u128),
        Decimal::from_str("340282366920938463463").unwrap()
    );
}

#[test]
#[should_panic(expected = "Denominator must not be zero")]
fn decimal_from_ratio_panics_for_zero_denominator() {
    Decimal::from_ratio(1u128, 0u128);
}

#[test]
#[should_panic(expected = "Multiplication overflow")]
fn decimal_from_ratio_panics_for_mul_overflow() {
    Decimal::from_ratio(u128::MAX, 1u128);
}

#[test]
fn decimal_checked_from_ratio_does_not_panic() {
    assert_eq!(
        Decimal::checked_from_ratio(1u128, 0u128),
        Err(CheckedFromRatioError::DivideByZero)
    );

    assert_eq!(
        Decimal::checked_from_ratio(u128::MAX, 1u128),
        Err(CheckedFromRatioError::Overflow)
    );
}

#[test]
fn decimal_implements_fraction() {
    let fraction = Decimal::from_str("1234.567").unwrap();
    assert_eq!(
        fraction.numerator(),
        Uint128::from(1_234_567_000_000_000_000_000u128)
    );
    assert_eq!(
        fraction.denominator(),
        Uint128::from(1_000_000_000_000_000_000u128)
    );
}

#[test]
fn decimal_from_str_works() {
    // Integers
    assert_eq!(Decimal::from_str("0").unwrap(), Decimal::percent(0));
    assert_eq!(Decimal::from_str("1").unwrap(), Decimal::percent(100));
    assert_eq!(Decimal::from_str("5").unwrap(), Decimal::percent(500));
    assert_eq!(Decimal::from_str("42").unwrap(), Decimal::percent(4200));
    assert_eq!(Decimal::from_str("000").unwrap(), Decimal::percent(0));
    assert_eq!(Decimal::from_str("001").unwrap(), Decimal::percent(100));
    assert_eq!(Decimal::from_str("005").unwrap(), Decimal::percent(500));
    assert_eq!(Decimal::from_str("0042").unwrap(), Decimal::percent(4200));

    // Decimals
    assert_eq!(Decimal::from_str("1.0").unwrap(), Decimal::percent(100));
    assert_eq!(Decimal::from_str("1.5").unwrap(), Decimal::percent(150));
    assert_eq!(Decimal::from_str("0.5").unwrap(), Decimal::percent(50));
    assert_eq!(Decimal::from_str("0.123").unwrap(), Decimal::permille(123));

    assert_eq!(Decimal::from_str("40.00").unwrap(), Decimal::percent(4000));
    assert_eq!(Decimal::from_str("04.00").unwrap(), Decimal::percent(400));
    assert_eq!(Decimal::from_str("00.40").unwrap(), Decimal::percent(40));
    assert_eq!(Decimal::from_str("00.04").unwrap(), Decimal::percent(4));

    // Can handle DECIMAL_PLACES fractional digits
    assert_eq!(
        Decimal::from_str("7.123456789012345678").unwrap(),
        Decimal::new(Uint128::from(7123456789012345678u128))
    );
    assert_eq!(
        Decimal::from_str("7.999999999999999999").unwrap(),
        Decimal::new(Uint128::from(7999999999999999999u128))
    );

    // Works for documented max value
    assert_eq!(
        Decimal::from_str("340282366920938463463.374607431768211455").unwrap(),
        Decimal::MAX
    );
}

#[test]
fn decimal_from_str_errors_for_broken_whole_part() {
    assert!(Decimal::from_str("").is_err());
    assert!(Decimal::from_str(" ").is_err());
    assert!(Decimal::from_str("-1").is_err());
}

#[test]
fn decimal_from_str_errors_for_broken_fractional_part() {
    assert!(Decimal::from_str("1.").is_err());
    assert!(Decimal::from_str("1. ").is_err());
    assert!(Decimal::from_str("1.e").is_err());
    assert!(Decimal::from_str("1.2e3").is_err());
}

#[test]
fn decimal_from_str_errors_for_more_than_18_fractional_digits() {
    assert!(Decimal::from_str("7.1234567890123456789")
        .unwrap_err()
        .to_string()
        .ends_with("Cannot parse more than 18 fractional digits"));

    // No special rules for trailing zeros. This could be changed but adds gas cost for the happy path.
    assert!(Decimal::from_str("7.1230000000000000000")
        .unwrap_err()
        .to_string()
        .ends_with("Cannot parse more than 18 fractional digits"));
}

#[test]
fn decimal_from_str_errors_for_invalid_number_of_dots() {
    assert!(Decimal::from_str("1.2.3").is_err());
    assert!(Decimal::from_str("1.2.3.4").is_err());
}

#[test]
fn decimal_from_str_errors_for_more_than_max_value() {
    // Integer
    assert!(Decimal::from_str("340282366920938463464").is_err());

    // Decimal
    assert!(Decimal::from_str("340282366920938463464.0").is_err());
    assert!(Decimal::from_str("340282366920938463463.374607431768211456").is_err());
}

#[test]
fn decimal_atomics_works() {
    let zero = Decimal::zero();
    let one = Decimal::one();
    let half = Decimal::percent(50);
    let two = Decimal::percent(200);
    let max = Decimal::MAX;

    assert_eq!(zero.atomics(), Uint128::new(0));
    assert_eq!(one.atomics(), Uint128::new(1000000000000000000));
    assert_eq!(half.atomics(), Uint128::new(500000000000000000));
    assert_eq!(two.atomics(), Uint128::new(2000000000000000000));
    assert_eq!(max.atomics(), Uint128::MAX);
}

#[test]
fn decimal_decimal_places_works() {
    let zero = Decimal::zero();
    let one = Decimal::one();
    let half = Decimal::percent(50);
    let two = Decimal::percent(200);
    let max = Decimal::MAX;

    assert_eq!(zero.decimal_places(), 18);
    assert_eq!(one.decimal_places(), 18);
    assert_eq!(half.decimal_places(), 18);
    assert_eq!(two.decimal_places(), 18);
    assert_eq!(max.decimal_places(), 18);
}

#[test]
fn decimal_is_zero_works() {
    assert!(Decimal::zero().is_zero());
    assert!(Decimal::percent(0).is_zero());
    assert!(Decimal::permille(0).is_zero());

    assert!(!Decimal::one().is_zero());
    assert!(!Decimal::percent(123).is_zero());
    assert!(!Decimal::permille(1234).is_zero());
}

#[test]
fn decimal_inv_works() {
    // d = 0
    assert_eq!(Decimal::zero().inv(), None);

    // d == 1
    assert_eq!(Decimal::one().inv(), Some(Decimal::one()));

    // d > 1 exact
    assert_eq!(
        Decimal::from_str("2").unwrap().inv(),
        Some(Decimal::from_str("0.5").unwrap())
    );
    assert_eq!(
        Decimal::from_str("20").unwrap().inv(),
        Some(Decimal::from_str("0.05").unwrap())
    );
    assert_eq!(
        Decimal::from_str("200").unwrap().inv(),
        Some(Decimal::from_str("0.005").unwrap())
    );
    assert_eq!(
        Decimal::from_str("2000").unwrap().inv(),
        Some(Decimal::from_str("0.0005").unwrap())
    );

    // d > 1 rounded
    assert_eq!(
        Decimal::from_str("3").unwrap().inv(),
        Some(Decimal::from_str("0.333333333333333333").unwrap())
    );
    assert_eq!(
        Decimal::from_str("6").unwrap().inv(),
        Some(Decimal::from_str("0.166666666666666666").unwrap())
    );

    // d < 1 exact
    assert_eq!(
        Decimal::from_str("0.5").unwrap().inv(),
        Some(Decimal::from_str("2").unwrap())
    );
    assert_eq!(
        Decimal::from_str("0.05").unwrap().inv(),
        Some(Decimal::from_str("20").unwrap())
    );
    assert_eq!(
        Decimal::from_str("0.005").unwrap().inv(),
        Some(Decimal::from_str("200").unwrap())
    );
    assert_eq!(
        Decimal::from_str("0.0005").unwrap().inv(),
        Some(Decimal::from_str("2000").unwrap())
    );
}

#[test]
#[allow(clippy::op_ref)]
fn decimal_add_works() {
    let value = Decimal::one() + Decimal::percent(50); // 1.5
    assert_eq!(
        value,
        Decimal::new(DECIMAL_FRACTIONAL * Uint128::from(3u8) / Uint128::from(2u8))
    );

    assert_eq!(
        Decimal::percent(5) + Decimal::percent(4),
        Decimal::percent(9)
    );
    assert_eq!(Decimal::percent(5) + Decimal::zero(), Decimal::percent(5));
    assert_eq!(Decimal::zero() + Decimal::zero(), Decimal::zero());

    // works for refs
    let a = Decimal::percent(15);
    let b = Decimal::percent(25);
    let expected = Decimal::percent(40);
    assert_eq!(a + b, expected);
    assert_eq!(&a + b, expected);
    assert_eq!(a + &b, expected);
    assert_eq!(&a + &b, expected);
}

#[test]
#[should_panic(expected = "attempt to add with overflow")]
fn decimal_add_overflow_panics() {
    let _value = Decimal::MAX + Decimal::percent(50);
}

#[test]
fn decimal_add_assign_works() {
    let mut a = Decimal::percent(30);
    a += Decimal::percent(20);
    assert_eq!(a, Decimal::percent(50));

    // works for refs
    let mut a = Decimal::percent(15);
    let b = Decimal::percent(3);
    let expected = Decimal::percent(18);
    a += &b;
    assert_eq!(a, expected);
}

#[test]
#[allow(clippy::op_ref)]
fn decimal_sub_works() {
    let value = Decimal::one() - Decimal::percent(50); // 0.5
    assert_eq!(value, Decimal::new(DECIMAL_FRACTIONAL / Uint128::from(2u8)));

    assert_eq!(
        Decimal::percent(9) - Decimal::percent(4),
        Decimal::percent(5)
    );
    assert_eq!(Decimal::percent(16) - Decimal::zero(), Decimal::percent(16));
    assert_eq!(Decimal::percent(16) - Decimal::percent(16), Decimal::zero());
    assert_eq!(Decimal::zero() - Decimal::zero(), Decimal::zero());

    // works for refs
    let a = Decimal::percent(13);
    let b = Decimal::percent(6);
    let expected = Decimal::percent(7);
    assert_eq!(a - b, expected);
    assert_eq!(&a - b, expected);
    assert_eq!(a - &b, expected);
    assert_eq!(&a - &b, expected);
}

#[test]
#[should_panic(expected = "attempt to subtract with overflow")]
fn decimal_sub_overflow_panics() {
    let _value = Decimal::zero() - Decimal::percent(50);
}

#[test]
fn decimal_sub_assign_works() {
    let mut a = Decimal::percent(20);
    a -= Decimal::percent(2);
    assert_eq!(a, Decimal::percent(18));

    // works for refs
    let mut a = Decimal::percent(33);
    let b = Decimal::percent(13);
    let expected = Decimal::percent(20);
    a -= &b;
    assert_eq!(a, expected);
}

#[test]
#[allow(clippy::op_ref)]
fn decimal_implements_mul() {
    let one = Decimal::one();
    let two = one + one;
    let half = Decimal::percent(50);

    // 1*x and x*1
    assert_eq!(one * Decimal::percent(0), Decimal::percent(0));
    assert_eq!(one * Decimal::percent(1), Decimal::percent(1));
    assert_eq!(one * Decimal::percent(10), Decimal::percent(10));
    assert_eq!(one * Decimal::percent(100), Decimal::percent(100));
    assert_eq!(one * Decimal::percent(1000), Decimal::percent(1000));
    assert_eq!(one * Decimal::MAX, Decimal::MAX);
    assert_eq!(Decimal::percent(0) * one, Decimal::percent(0));
    assert_eq!(Decimal::percent(1) * one, Decimal::percent(1));
    assert_eq!(Decimal::percent(10) * one, Decimal::percent(10));
    assert_eq!(Decimal::percent(100) * one, Decimal::percent(100));
    assert_eq!(Decimal::percent(1000) * one, Decimal::percent(1000));
    assert_eq!(Decimal::MAX * one, Decimal::MAX);

    // double
    assert_eq!(two * Decimal::percent(0), Decimal::percent(0));
    assert_eq!(two * Decimal::percent(1), Decimal::percent(2));
    assert_eq!(two * Decimal::percent(10), Decimal::percent(20));
    assert_eq!(two * Decimal::percent(100), Decimal::percent(200));
    assert_eq!(two * Decimal::percent(1000), Decimal::percent(2000));
    assert_eq!(Decimal::percent(0) * two, Decimal::percent(0));
    assert_eq!(Decimal::percent(1) * two, Decimal::percent(2));
    assert_eq!(Decimal::percent(10) * two, Decimal::percent(20));
    assert_eq!(Decimal::percent(100) * two, Decimal::percent(200));
    assert_eq!(Decimal::percent(1000) * two, Decimal::percent(2000));

    // half
    assert_eq!(half * Decimal::percent(0), Decimal::percent(0));
    assert_eq!(half * Decimal::percent(1), Decimal::permille(5));
    assert_eq!(half * Decimal::percent(10), Decimal::percent(5));
    assert_eq!(half * Decimal::percent(100), Decimal::percent(50));
    assert_eq!(half * Decimal::percent(1000), Decimal::percent(500));
    assert_eq!(Decimal::percent(0) * half, Decimal::percent(0));
    assert_eq!(Decimal::percent(1) * half, Decimal::permille(5));
    assert_eq!(Decimal::percent(10) * half, Decimal::percent(5));
    assert_eq!(Decimal::percent(100) * half, Decimal::percent(50));
    assert_eq!(Decimal::percent(1000) * half, Decimal::percent(500));

    // Move left
    let a = dec("123.127726548762582");
    assert_eq!(a * dec("1"), dec("123.127726548762582"));
    assert_eq!(a * dec("10"), dec("1231.27726548762582"));
    assert_eq!(a * dec("100"), dec("12312.7726548762582"));
    assert_eq!(a * dec("1000"), dec("123127.726548762582"));
    assert_eq!(a * dec("1000000"), dec("123127726.548762582"));
    assert_eq!(a * dec("1000000000"), dec("123127726548.762582"));
    assert_eq!(a * dec("1000000000000"), dec("123127726548762.582"));
    assert_eq!(a * dec("1000000000000000"), dec("123127726548762582"));
    assert_eq!(a * dec("1000000000000000000"), dec("123127726548762582000"));
    assert_eq!(dec("1") * a, dec("123.127726548762582"));
    assert_eq!(dec("10") * a, dec("1231.27726548762582"));
    assert_eq!(dec("100") * a, dec("12312.7726548762582"));
    assert_eq!(dec("1000") * a, dec("123127.726548762582"));
    assert_eq!(dec("1000000") * a, dec("123127726.548762582"));
    assert_eq!(dec("1000000000") * a, dec("123127726548.762582"));
    assert_eq!(dec("1000000000000") * a, dec("123127726548762.582"));
    assert_eq!(dec("1000000000000000") * a, dec("123127726548762582"));
    assert_eq!(dec("1000000000000000000") * a, dec("123127726548762582000"));

    // Move right
    let max = Decimal::MAX;
    assert_eq!(
        max * dec("1.0"),
        dec("340282366920938463463.374607431768211455")
    );
    assert_eq!(
        max * dec("0.1"),
        dec("34028236692093846346.337460743176821145")
    );
    assert_eq!(
        max * dec("0.01"),
        dec("3402823669209384634.633746074317682114")
    );
    assert_eq!(
        max * dec("0.001"),
        dec("340282366920938463.463374607431768211")
    );
    assert_eq!(
        max * dec("0.000001"),
        dec("340282366920938.463463374607431768")
    );
    assert_eq!(
        max * dec("0.000000001"),
        dec("340282366920.938463463374607431")
    );
    assert_eq!(
        max * dec("0.000000000001"),
        dec("340282366.920938463463374607")
    );
    assert_eq!(
        max * dec("0.000000000000001"),
        dec("340282.366920938463463374")
    );
    assert_eq!(
        max * dec("0.000000000000000001"),
        dec("340.282366920938463463")
    );

    // works for refs
    let a = Decimal::percent(20);
    let b = Decimal::percent(30);
    let expected = Decimal::percent(6);
    assert_eq!(a * b, expected);
    assert_eq!(&a * b, expected);
    assert_eq!(a * &b, expected);
    assert_eq!(&a * &b, expected);
}

#[test]
fn decimal_mul_assign_works() {
    let mut a = Decimal::percent(15);
    a *= Decimal::percent(60);
    assert_eq!(a, Decimal::percent(9));

    // works for refs
    let mut a = Decimal::percent(50);
    let b = Decimal::percent(20);
    a *= &b;
    assert_eq!(a, Decimal::percent(10));
}

#[test]
#[should_panic(expected = "attempt to multiply with overflow")]
fn decimal_mul_overflow_panics() {
    let _value = Decimal::MAX * Decimal::percent(101);
}

#[test]
fn decimal_checked_mul() {
    let test_data = [
        (Decimal::zero(), Decimal::zero()),
        (Decimal::zero(), Decimal::one()),
        (Decimal::one(), Decimal::zero()),
        (Decimal::percent(10), Decimal::zero()),
        (Decimal::percent(10), Decimal::percent(5)),
        (Decimal::MAX, Decimal::one()),
        (Decimal::MAX / Uint128::new(2), Decimal::percent(200)),
        (Decimal::permille(6), Decimal::permille(13)),
    ];

    // The regular core::ops::Mul is our source of truth for these tests.
    for (x, y) in test_data.into_iter() {
        assert_eq!(x * y, x.checked_mul(y).unwrap());
    }
}

#[test]
fn decimal_checked_mul_overflow() {
    assert_eq!(
        Decimal::MAX.checked_mul(Decimal::percent(200)),
        Err(OverflowError::new(OverflowOperation::Mul))
    );
}

#[test]
#[allow(clippy::op_ref)]
fn decimal_implements_div() {
    let one = Decimal::one();
    let two = one + one;
    let half = Decimal::percent(50);

    // 1/x and x/1
    assert_eq!(one / Decimal::percent(1), Decimal::percent(10_000));
    assert_eq!(one / Decimal::percent(10), Decimal::percent(1_000));
    assert_eq!(one / Decimal::percent(100), Decimal::percent(100));
    assert_eq!(one / Decimal::percent(1000), Decimal::percent(10));
    assert_eq!(Decimal::percent(0) / one, Decimal::percent(0));
    assert_eq!(Decimal::percent(1) / one, Decimal::percent(1));
    assert_eq!(Decimal::percent(10) / one, Decimal::percent(10));
    assert_eq!(Decimal::percent(100) / one, Decimal::percent(100));
    assert_eq!(Decimal::percent(1000) / one, Decimal::percent(1000));

    // double
    assert_eq!(two / Decimal::percent(1), Decimal::percent(20_000));
    assert_eq!(two / Decimal::percent(10), Decimal::percent(2_000));
    assert_eq!(two / Decimal::percent(100), Decimal::percent(200));
    assert_eq!(two / Decimal::percent(1000), Decimal::percent(20));
    assert_eq!(Decimal::percent(0) / two, Decimal::percent(0));
    assert_eq!(Decimal::percent(1) / two, dec("0.005"));
    assert_eq!(Decimal::percent(10) / two, Decimal::percent(5));
    assert_eq!(Decimal::percent(100) / two, Decimal::percent(50));
    assert_eq!(Decimal::percent(1000) / two, Decimal::percent(500));

    // half
    assert_eq!(half / Decimal::percent(1), Decimal::percent(5_000));
    assert_eq!(half / Decimal::percent(10), Decimal::percent(500));
    assert_eq!(half / Decimal::percent(100), Decimal::percent(50));
    assert_eq!(half / Decimal::percent(1000), Decimal::percent(5));
    assert_eq!(Decimal::percent(0) / half, Decimal::percent(0));
    assert_eq!(Decimal::percent(1) / half, Decimal::percent(2));
    assert_eq!(Decimal::percent(10) / half, Decimal::percent(20));
    assert_eq!(Decimal::percent(100) / half, Decimal::percent(200));
    assert_eq!(Decimal::percent(1000) / half, Decimal::percent(2000));

    // Move right
    let a = dec("123127726548762582");
    assert_eq!(a / dec("1"), dec("123127726548762582"));
    assert_eq!(a / dec("10"), dec("12312772654876258.2"));
    assert_eq!(a / dec("100"), dec("1231277265487625.82"));
    assert_eq!(a / dec("1000"), dec("123127726548762.582"));
    assert_eq!(a / dec("1000000"), dec("123127726548.762582"));
    assert_eq!(a / dec("1000000000"), dec("123127726.548762582"));
    assert_eq!(a / dec("1000000000000"), dec("123127.726548762582"));
    assert_eq!(a / dec("1000000000000000"), dec("123.127726548762582"));
    assert_eq!(a / dec("1000000000000000000"), dec("0.123127726548762582"));
    assert_eq!(dec("1") / a, dec("0.000000000000000008"));
    assert_eq!(dec("10") / a, dec("0.000000000000000081"));
    assert_eq!(dec("100") / a, dec("0.000000000000000812"));
    assert_eq!(dec("1000") / a, dec("0.000000000000008121"));
    assert_eq!(dec("1000000") / a, dec("0.000000000008121647"));
    assert_eq!(dec("1000000000") / a, dec("0.000000008121647560"));
    assert_eq!(dec("1000000000000") / a, dec("0.000008121647560868"));
    assert_eq!(dec("1000000000000000") / a, dec("0.008121647560868164"));
    assert_eq!(dec("1000000000000000000") / a, dec("8.121647560868164773"));

    // Move left
    let a = dec("0.123127726548762582");
    assert_eq!(a / dec("1.0"), dec("0.123127726548762582"));
    assert_eq!(a / dec("0.1"), dec("1.23127726548762582"));
    assert_eq!(a / dec("0.01"), dec("12.3127726548762582"));
    assert_eq!(a / dec("0.001"), dec("123.127726548762582"));
    assert_eq!(a / dec("0.000001"), dec("123127.726548762582"));
    assert_eq!(a / dec("0.000000001"), dec("123127726.548762582"));
    assert_eq!(a / dec("0.000000000001"), dec("123127726548.762582"));
    assert_eq!(a / dec("0.000000000000001"), dec("123127726548762.582"));
    assert_eq!(a / dec("0.000000000000000001"), dec("123127726548762582"));

    assert_eq!(
        Decimal::percent(15) / Decimal::percent(60),
        Decimal::percent(25)
    );

    // works for refs
    let a = Decimal::percent(100);
    let b = Decimal::percent(20);
    let expected = Decimal::percent(500);
    assert_eq!(a / b, expected);
    assert_eq!(&a / b, expected);
    assert_eq!(a / &b, expected);
    assert_eq!(&a / &b, expected);
}

#[test]
fn decimal_div_assign_works() {
    let mut a = Decimal::percent(15);
    a /= Decimal::percent(20);
    assert_eq!(a, Decimal::percent(75));

    // works for refs
    let mut a = Decimal::percent(50);
    let b = Decimal::percent(20);
    a /= &b;
    assert_eq!(a, Decimal::percent(250));
}

#[test]
#[should_panic(expected = "Division failed - multiplication overflow")]
fn decimal_div_overflow_panics() {
    let _value = Decimal::MAX / Decimal::percent(10);
}

#[test]
#[should_panic(expected = "Division failed - denominator must not be zero")]
fn decimal_div_by_zero_panics() {
    let _value = Decimal::one() / Decimal::zero();
}

#[test]
fn decimal_uint128_division() {
    // a/b
    let left = Decimal::percent(150); // 1.5
    let right = Uint128::new(3);
    assert_eq!(left / right, Decimal::percent(50));

    // 0/a
    let left = Decimal::zero();
    let right = Uint128::new(300);
    assert_eq!(left / right, Decimal::zero());
}

#[test]
#[should_panic(expected = "attempt to divide by zero")]
fn decimal_uint128_divide_by_zero() {
    let left = Decimal::percent(150); // 1.5
    let right = Uint128::new(0);
    let _result = left / right;
}

#[test]
fn decimal_uint128_div_assign() {
    // a/b
    let mut dec = Decimal::percent(150); // 1.5
    dec /= Uint128::new(3);
    assert_eq!(dec, Decimal::percent(50));

    // 0/a
    let mut dec = Decimal::zero();
    dec /= Uint128::new(300);
    assert_eq!(dec, Decimal::zero());
}

#[test]
#[should_panic(expected = "attempt to divide by zero")]
fn decimal_uint128_div_assign_by_zero() {
    // a/0
    let mut dec = Decimal::percent(50);
    dec /= Uint128::new(0);
}

#[test]
fn decimal_uint128_sqrt() {
    assert_eq!(Decimal::percent(0).sqrt(), Decimal::percent(0));
    assert_eq!(Decimal::percent(900).sqrt(), Decimal::percent(300));
    assert!(Decimal::percent(316) < Decimal::percent(1000).sqrt());
    assert!(Decimal::percent(1000).sqrt() < Decimal::percent(317));
}

/// sqrt(2) is an irrational number, i.e. all 18 decimal places should be used.
#[test]
fn decimal_uint128_sqrt_is_precise() {
    assert_eq!(
        Decimal::from_str("2").unwrap().sqrt(),
        Decimal::from_str("1.414213562373095048").unwrap() // https://www.wolframalpha.com/input/?i=sqrt%282%29
    );
}

#[test]
fn decimal_uint128_sqrt_does_not_overflow() {
    assert_eq!(
        Decimal::from_str("400").unwrap().sqrt(),
        Decimal::from_str("20").unwrap()
    );
}

#[test]
fn decimal_uint128_sqrt_intermediate_precision_used() {
    assert_eq!(
        Decimal::from_str("400001").unwrap().sqrt(),
        // The last two digits (27) are truncated below due to the algorithm
        // we use. Larger numbers will cause less precision.
        // https://www.wolframalpha.com/input/?i=sqrt%28400001%29
        Decimal::from_str("632.456322602596803200").unwrap()
    );
}

#[test]
fn decimal_checked_pow() {
    for exp in 0..10 {
        assert_eq!(Decimal::one().checked_pow(exp).unwrap(), Decimal::one());
    }

    // This case is mathematically undefined, but we ensure consistency with Rust standard types
    // https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=20df6716048e77087acd40194b233494
    assert_eq!(Decimal::zero().checked_pow(0).unwrap(), Decimal::one());

    for exp in 1..10 {
        assert_eq!(Decimal::zero().checked_pow(exp).unwrap(), Decimal::zero());
    }

    for num in &[
        Decimal::percent(50),
        Decimal::percent(99),
        Decimal::percent(200),
    ] {
        assert_eq!(num.checked_pow(0).unwrap(), Decimal::one())
    }

    assert_eq!(
        Decimal::percent(20).checked_pow(2).unwrap(),
        Decimal::percent(4)
    );

    assert_eq!(
        Decimal::percent(20).checked_pow(3).unwrap(),
        Decimal::permille(8)
    );

    assert_eq!(
        Decimal::percent(200).checked_pow(4).unwrap(),
        Decimal::percent(1600)
    );

    assert_eq!(
        Decimal::percent(200).checked_pow(4).unwrap(),
        Decimal::percent(1600)
    );

    assert_eq!(
        Decimal::percent(700).checked_pow(5).unwrap(),
        Decimal::percent(1680700)
    );

    assert_eq!(
        Decimal::percent(700).checked_pow(8).unwrap(),
        Decimal::percent(576480100)
    );

    assert_eq!(
        Decimal::percent(700).checked_pow(10).unwrap(),
        Decimal::percent(28247524900)
    );

    assert_eq!(
        Decimal::percent(120).checked_pow(123).unwrap(),
        Decimal::new(5486473221892422150877397607u128.into())
    );

    assert_eq!(
        Decimal::percent(10).checked_pow(2).unwrap(),
        Decimal::new(10000000000000000u128.into())
    );

    assert_eq!(
        Decimal::percent(10).checked_pow(18).unwrap(),
        Decimal::new(1u128.into())
    );
}

#[test]
fn decimal_checked_pow_overflow() {
    assert_eq!(
        Decimal::MAX.checked_pow(2),
        Err(OverflowError::new(OverflowOperation::Pow))
    );
    assert_eq!(
        Decimal::MAX.checked_pow(3),
        Err(OverflowError::new(OverflowOperation::Pow))
    );
    assert_eq!(
        Decimal::new(DECIMAL_FRACTIONAL * Uint128::new(1_000_000_000)).checked_pow(15),
        Err(OverflowError::new(OverflowOperation::Pow))
    );
}

#[test]
fn decimal_to_string() {
    // Integers
    assert_eq!(Decimal::zero().to_string(), "0");
    assert_eq!(Decimal::one().to_string(), "1");
    assert_eq!(Decimal::percent(500).to_string(), "5");

    // Decimals
    assert_eq!(Decimal::percent(125).to_string(), "1.25");
    assert_eq!(Decimal::percent(42638).to_string(), "426.38");
    assert_eq!(Decimal::percent(3).to_string(), "0.03");
    assert_eq!(Decimal::permille(987).to_string(), "0.987");

    assert_eq!(
        Decimal::new(Uint128::from(1u128)).to_string(),
        "0.000000000000000001"
    );
    assert_eq!(
        Decimal::new(Uint128::from(10u128)).to_string(),
        "0.00000000000000001"
    );
    assert_eq!(
        Decimal::new(Uint128::from(100u128)).to_string(),
        "0.0000000000000001"
    );
    assert_eq!(
        Decimal::new(Uint128::from(1000u128)).to_string(),
        "0.000000000000001"
    );
    assert_eq!(
        Decimal::new(Uint128::from(10000u128)).to_string(),
        "0.00000000000001"
    );
    assert_eq!(
        Decimal::new(Uint128::from(100000u128)).to_string(),
        "0.0000000000001"
    );
    assert_eq!(
        Decimal::new(Uint128::from(1000000u128)).to_string(),
        "0.000000000001"
    );
    assert_eq!(
        Decimal::new(Uint128::from(10000000u128)).to_string(),
        "0.00000000001"
    );
    assert_eq!(
        Decimal::new(Uint128::from(100000000u128)).to_string(),
        "0.0000000001"
    );
    assert_eq!(
        Decimal::new(Uint128::from(1000000000u128)).to_string(),
        "0.000000001"
    );
    assert_eq!(
        Decimal::new(Uint128::from(10000000000u128)).to_string(),
        "0.00000001"
    );
    assert_eq!(
        Decimal::new(Uint128::from(100000000000u128)).to_string(),
        "0.0000001"
    );
    assert_eq!(
        Decimal::new(Uint128::from(10000000000000u128)).to_string(),
        "0.00001"
    );
    assert_eq!(
        Decimal::new(Uint128::from(100000000000000u128)).to_string(),
        "0.0001"
    );
    assert_eq!(
        Decimal::new(Uint128::from(1000000000000000u128)).to_string(),
        "0.001"
    );
    assert_eq!(
        Decimal::new(Uint128::from(10000000000000000u128)).to_string(),
        "0.01"
    );
    assert_eq!(
        Decimal::new(Uint128::from(100000000000000000u128)).to_string(),
        "0.1"
    );
}

#[test]
fn decimal_iter_sum() {
    let items = vec![
        Decimal::zero(),
        Decimal::new(Uint128::from(2u128)),
        Decimal::new(Uint128::from(2u128)),
    ];
    assert_eq!(
        items.iter().sum::<Decimal>(),
        Decimal::new(Uint128::from(4u128))
    );
    assert_eq!(
        items.into_iter().sum::<Decimal>(),
        Decimal::new(Uint128::from(4u128))
    );

    let empty: Vec<Decimal> = vec![];
    assert_eq!(Decimal::zero(), empty.iter().sum::<Decimal>());
}

#[test]
fn decimal_serialize() {
    assert_eq!(serde_json::to_vec(&Decimal::zero()).unwrap(), br#""0""#);
    assert_eq!(serde_json::to_vec(&Decimal::one()).unwrap(), br#""1""#);
    assert_eq!(
        serde_json::to_vec(&Decimal::percent(8)).unwrap(),
        br#""0.08""#
    );
    assert_eq!(
        serde_json::to_vec(&Decimal::percent(87)).unwrap(),
        br#""0.87""#
    );
    assert_eq!(
        serde_json::to_vec(&Decimal::percent(876)).unwrap(),
        br#""8.76""#
    );
    assert_eq!(
        serde_json::to_vec(&Decimal::percent(8765)).unwrap(),
        br#""87.65""#
    );
}

#[test]
fn decimal_deserialize() {
    assert_eq!(
        serde_json::from_slice::<Decimal>(br#""0""#).unwrap(),
        Decimal::zero()
    );
    assert_eq!(
        serde_json::from_slice::<Decimal>(br#""1""#).unwrap(),
        Decimal::one()
    );
    assert_eq!(
        serde_json::from_slice::<Decimal>(br#""000""#).unwrap(),
        Decimal::zero()
    );
    assert_eq!(
        serde_json::from_slice::<Decimal>(br#""001""#).unwrap(),
        Decimal::one()
    );

    assert_eq!(
        serde_json::from_slice::<Decimal>(br#""0.08""#).unwrap(),
        Decimal::percent(8)
    );
    assert_eq!(
        serde_json::from_slice::<Decimal>(br#""0.87""#).unwrap(),
        Decimal::percent(87)
    );
    assert_eq!(
        serde_json::from_slice::<Decimal>(br#""8.76""#).unwrap(),
        Decimal::percent(876)
    );
    assert_eq!(
        serde_json::from_slice::<Decimal>(br#""87.65""#).unwrap(),
        Decimal::percent(8765)
    );
}

#[test]
fn decimal_abs_diff_works() {
    let a = Decimal::percent(285);
    let b = Decimal::percent(200);
    let expected = Decimal::percent(85);
    assert_eq!(a.abs_diff(b), expected);
    assert_eq!(b.abs_diff(a), expected);
}

#[test]
#[allow(clippy::op_ref)]
fn decimal_rem_works() {
    // 4.02 % 1.11 = 0.69
    assert_eq!(
        Decimal::percent(402) % Decimal::percent(111),
        Decimal::percent(69)
    );

    // 15.25 % 4 = 3.25
    assert_eq!(
        Decimal::percent(1525) % Decimal::percent(400),
        Decimal::percent(325)
    );

    let a = Decimal::percent(318);
    let b = Decimal::percent(317);
    let expected = Decimal::percent(1);
    assert_eq!(a % b, expected);
    assert_eq!(a % &b, expected);
    assert_eq!(&a % b, expected);
    assert_eq!(&a % &b, expected);
}

#[test]
fn decimal_rem_assign_works() {
    let mut a = Decimal::percent(17673);
    a %= Decimal::percent(2362);
    assert_eq!(a, Decimal::percent(1139)); // 176.73 % 23.62 = 11.39

    let mut a = Decimal::percent(4262);
    let b = Decimal::percent(1270);
    a %= &b;
    assert_eq!(a, Decimal::percent(452)); // 42.62 % 12.7 = 4.52
}

#[test]
#[should_panic(expected = "divisor of zero")]
fn decimal_rem_panics_for_zero() {
    let _ = Decimal::percent(777) % Decimal::zero();
}

#[test]
fn decimal_checked_methods() {
    // checked add
    assert_eq!(
        Decimal::percent(402)
            .checked_add(Decimal::percent(111))
            .unwrap(),
        Decimal::percent(513)
    );
    assert!(matches!(
        Decimal::MAX.checked_add(Decimal::percent(1)),
        Err(OverflowError { .. })
    ));

    // checked sub
    assert_eq!(
        Decimal::percent(1111)
            .checked_sub(Decimal::percent(111))
            .unwrap(),
        Decimal::percent(1000)
    );
    assert!(matches!(
        Decimal::zero().checked_sub(Decimal::percent(1)),
        Err(OverflowError { .. })
    ));

    // checked div
    assert_eq!(
        Decimal::percent(30)
            .checked_div(Decimal::percent(200))
            .unwrap(),
        Decimal::percent(15)
    );
    assert_eq!(
        Decimal::percent(88)
            .checked_div(Decimal::percent(20))
            .unwrap(),
        Decimal::percent(440)
    );
    assert!(matches!(
        Decimal::MAX.checked_div(Decimal::zero()),
        Err(CheckedFromRatioError::DivideByZero)
    ));
    assert!(matches!(
        Decimal::MAX.checked_div(Decimal::percent(1)),
        Err(CheckedFromRatioError::Overflow)
    ));

    // checked rem
    assert_eq!(
        Decimal::percent(402)
            .checked_rem(Decimal::percent(111))
            .unwrap(),
        Decimal::percent(69)
    );
    assert_eq!(
        Decimal::percent(1525)
            .checked_rem(Decimal::percent(400))
            .unwrap(),
        Decimal::percent(325)
    );
    assert!(matches!(
        Decimal::MAX.checked_rem(Decimal::zero()),
        Err(DivideByZeroError { .. })
    ));
}

#[test]
fn decimal_pow_works() {
    assert_eq!(Decimal::percent(200).pow(2), Decimal::percent(400));
    assert_eq!(Decimal::percent(200).pow(10), Decimal::percent(102400));
}

#[test]
#[should_panic]
fn decimal_pow_overflow_panics() {
    _ = Decimal::MAX.pow(2u32);
}

#[test]
fn decimal_saturating_works() {
    assert_eq!(
        Decimal::percent(200).saturating_add(Decimal::percent(200)),
        Decimal::percent(400)
    );
    assert_eq!(
        Decimal::MAX.saturating_add(Decimal::percent(200)),
        Decimal::MAX
    );
    assert_eq!(
        Decimal::percent(200).saturating_sub(Decimal::percent(100)),
        Decimal::percent(100)
    );
    assert_eq!(
        Decimal::zero().saturating_sub(Decimal::percent(200)),
        Decimal::zero()
    );
    assert_eq!(
        Decimal::percent(200).saturating_mul(Decimal::percent(50)),
        Decimal::percent(100)
    );
    assert_eq!(
        Decimal::MAX.saturating_mul(Decimal::percent(200)),
        Decimal::MAX
    );
    assert_eq!(
        Decimal::percent(400).saturating_pow(2u32),
        Decimal::percent(1600)
    );
    assert_eq!(Decimal::MAX.saturating_pow(2u32), Decimal::MAX);
}

#[test]
fn decimal_rounding() {
    assert_eq!(Decimal::one().floor(), Decimal::one());
    assert_eq!(Decimal::percent(150).floor(), Decimal::one());
    assert_eq!(Decimal::percent(199).floor(), Decimal::one());
    assert_eq!(Decimal::percent(200).floor(), Decimal::percent(200));
    assert_eq!(Decimal::percent(99).floor(), Decimal::zero());

    assert_eq!(Decimal::one().ceil(), Decimal::one());
    assert_eq!(Decimal::percent(150).ceil(), Decimal::percent(200));
    assert_eq!(Decimal::percent(199).ceil(), Decimal::percent(200));
    assert_eq!(Decimal::percent(99).ceil(), Decimal::one());
    assert_eq!(Decimal::new(Uint128::from(1u128)).ceil(), Decimal::one());
}

#[test]
#[should_panic(expected = "attempt to ceil with overflow")]
fn decimal_ceil_panics() {
    let _ = Decimal::MAX.ceil();
}

#[test]
fn decimal_checked_ceil() {
    assert_eq!(
        Decimal::percent(199).checked_ceil(),
        Ok(Decimal::percent(200))
    );
    assert!(matches!(
        Decimal::MAX.checked_ceil(),
        Err(RoundUpOverflowError { .. })
    ));
}

#[test]
fn decimal_to_uint_floor_works() {
    let d = Decimal::from_str("12.000000000000000001").unwrap();
    assert_eq!(d.to_uint_floor(), Uint128::new(12));
    let d = Decimal::from_str("12.345").unwrap();
    assert_eq!(d.to_uint_floor(), Uint128::new(12));
    let d = Decimal::from_str("12.999").unwrap();
    assert_eq!(d.to_uint_floor(), Uint128::new(12));
    let d = Decimal::from_str("0.98451384").unwrap();
    assert_eq!(d.to_uint_floor(), Uint128::new(0));

    let d = Decimal::from_str("75.0").unwrap();
    assert_eq!(d.to_uint_floor(), Uint128::new(75));
    let d = Decimal::from_str("0.0").unwrap();
    assert_eq!(d.to_uint_floor(), Uint128::new(0));

    let d = Decimal::MAX;
    assert_eq!(d.to_uint_floor(), Uint128::new(340282366920938463463));

    // Does the same as the old workaround `Uint128::one() * my_decimal`.
    // This block can be deleted as part of https://github.com/CosmWasm/cosmwasm/issues/1485.
    let tests = vec![
        (Decimal::from_str("12.345").unwrap(), 12u128),
        (Decimal::from_str("0.98451384").unwrap(), 0u128),
        (Decimal::from_str("178.0").unwrap(), 178u128),
        (Decimal::MIN, 0u128),
        (Decimal::MAX, u128::MAX / DECIMAL_FRACTIONAL.u128()),
    ];
    for (my_decimal, expected) in tests.into_iter() {
        assert_eq!(my_decimal.to_uint_floor(), Uint128::new(expected));
    }
}

#[test]
fn decimal_to_uint_ceil_works() {
    let d = Decimal::from_str("12.000000000000000001").unwrap();
    assert_eq!(d.to_uint_ceil(), Uint128::new(13));
    let d = Decimal::from_str("12.345").unwrap();
    assert_eq!(d.to_uint_ceil(), Uint128::new(13));
    let d = Decimal::from_str("12.999").unwrap();
    assert_eq!(d.to_uint_ceil(), Uint128::new(13));

    let d = Decimal::from_str("75.0").unwrap();
    assert_eq!(d.to_uint_ceil(), Uint128::new(75));
    let d = Decimal::from_str("0.0").unwrap();
    assert_eq!(d.to_uint_ceil(), Uint128::new(0));

    let d = Decimal::MAX;
    assert_eq!(d.to_uint_ceil(), Uint128::new(340282366920938463464));
}

#[test]
fn decimal_partial_eq() {
    let test_cases = [
        ("1", "1", true),
        ("0.5", "0.5", true),
        ("0.5", "0.51", false),
        ("0", "0.00000", true),
    ]
    .into_iter()
    .map(|(lhs, rhs, expected)| (dec(lhs), dec(rhs), expected));

    #[allow(clippy::op_ref)]
    for (lhs, rhs, expected) in test_cases {
        assert_eq!(lhs == rhs, expected);
        assert_eq!(&lhs == rhs, expected);
        assert_eq!(lhs == &rhs, expected);
        assert_eq!(&lhs == &rhs, expected);
    }
}

#[test]
fn decimal_implements_debug() {
    let decimal = Decimal::from_str("123.45").unwrap();
    assert_eq!(format!("{decimal:?}"), "Decimal(123.45)");

    let test_cases = ["5", "5.01", "42", "0", "2"];
    for s in test_cases {
        let decimal = Decimal::from_str(s).unwrap();
        let expected = format!("Decimal({s})");
        assert_eq!(format!("{decimal:?}"), expected);
    }
}

#[test]
fn serialize_decimal_to_string() {
    let d = Decimal::from_str("123.45").unwrap();
    let json_string = serde_json::to_string(&d).unwrap();
    assert_eq!(r#""123.45""#, json_string);
}

#[test]
fn deserialize_decimal_from_string() {
    let json_string = r#""123.45""#;
    let d: Decimal = serde_json::from_str(json_string).unwrap();
    assert_eq!(d, Decimal::from_str("123.45").unwrap());
}

#[test]
fn deserialize_invalid_decimal() {
    let json_string = r#""123,45""#;
    let result: Result<Decimal, _> = serde_json::from_str(json_string);
    let err = result.unwrap_err();
    assert!(err.to_string().contains("invalid digit"));
}

#[test]
fn deserialize_wrong_type_triggers_expectation() {
    let json_decimal = "123.45";
    let err = serde_json::from_str::<Decimal>(json_decimal).unwrap_err();
    assert!(err.to_string().contains("expected string-encoded decimal"));
}

#[test]
fn failing_writer_should_work() {
    enum When {
        Always,
        OnDecimal,
        AfterDecimal,
    }
    struct FailingWriter {
        when: When,
        consumed_decimal: bool,
    }

    impl FailingWriter {
        fn new(when: When) -> Self {
            Self {
                when,
                consumed_decimal: false,
            }
        }
    }
    impl Write for FailingWriter {
        fn write_str(&mut self, s: &str) -> std::fmt::Result {
            match self.when {
                When::Always => return Err(std::fmt::Error),
                When::OnDecimal => {
                    if s == "." {
                        return Err(std::fmt::Error);
                    }
                }
                When::AfterDecimal => {
                    if self.consumed_decimal {
                        return Err(std::fmt::Error);
                    }
                    if s == "." {
                        self.consumed_decimal = true;
                    }
                }
            }
            Ok(())
        }
    }
    write!(
        &mut FailingWriter::new(When::Always),
        "{}",
        Decimal::from_str("123.456").unwrap()
    )
    .unwrap_err();

    write!(
        &mut FailingWriter::new(When::OnDecimal),
        "{}",
        Decimal::from_str("123.456").unwrap()
    )
    .unwrap_err();

    write!(
        &mut FailingWriter::new(When::AfterDecimal),
        "{}",
        Decimal::from_str("123.456").unwrap()
    )
    .unwrap_err();
}
