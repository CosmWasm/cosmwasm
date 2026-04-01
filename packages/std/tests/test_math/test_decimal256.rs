use cosmwasm_std::{
    CheckedFromRatioError, Decimal, Decimal256, Decimal256RangeExceeded, DivideByZeroError,
    Fraction, Int128, Int256, OverflowError, OverflowOperation, RoundUpOverflowError,
    SignedDecimal, SignedDecimal256, Uint128, Uint256,
};
use std::fmt::Write;
use std::str::FromStr;

/// 1*10**18
const DECIMAL_FRACTIONAL: Uint256 = Uint256::new(1_000_000_000_000_000_000);

fn dec(input: &str) -> Decimal256 {
    Decimal256::from_str(input).unwrap()
}

#[test]
fn decimal256_new() {
    let expected = Uint256::from(300u128);
    assert_eq!("0.0000000000000003", Decimal256::new(expected).to_string());
}

#[test]
#[allow(deprecated)]
fn decimal256_raw() {
    let value = 300u128;
    let expected = Uint256::from(value);
    assert_eq!(Decimal256::raw(value), Decimal256::new(expected));
}

#[test]
fn decimal256_one() {
    let value = Decimal256::one();
    assert_eq!(value, Decimal256::new(DECIMAL_FRACTIONAL));
}

#[test]
fn decimal256_zero() {
    let value = Decimal256::zero();
    assert!(value.is_zero());
}

#[test]
fn decimal256_percent() {
    let value = Decimal256::percent(50);
    assert_eq!(
        value,
        Decimal256::new(DECIMAL_FRACTIONAL / Uint256::from(2u8))
    );
}

#[test]
fn decimal256_permille() {
    let value = Decimal256::permille(125);
    assert_eq!(
        value,
        Decimal256::new(DECIMAL_FRACTIONAL / Uint256::from(8u8))
    );
}

#[test]
fn decimal256_bps() {
    let value = Decimal256::bps(125);
    assert_eq!(
        value,
        Decimal256::new(DECIMAL_FRACTIONAL / Uint256::from(80u8))
    );
}

#[test]
fn decimal256_from_decimal_works() {
    assert_eq!(
        Decimal256::from(Decimal::new(Uint128::MAX)),
        Decimal256::from(Decimal::MAX)
    );
    assert_eq!(Decimal256::from(Decimal::zero()), Decimal256::zero());
    assert_eq!(Decimal256::from(Decimal::one()), Decimal256::one());
    assert_eq!(
        Decimal256::from(Decimal::percent(50)),
        Decimal256::percent(50)
    );
}

#[test]
fn decimal256_from_signed_decimal_works() {
    assert_eq!(
        Decimal256::try_from(SignedDecimal::new(Int128::MAX)),
        Ok(Decimal256::new(Uint256::new(u128::MAX / 2)))
    );
    assert_eq!(
        Decimal256::try_from(SignedDecimal::new(Int128::MIN)),
        Err(Decimal256RangeExceeded)
    );
    assert_eq!(
        Decimal256::try_from(SignedDecimal::zero()),
        Ok(Decimal256::zero())
    );
    assert_eq!(
        Decimal256::try_from(SignedDecimal::one()),
        Ok(Decimal256::one())
    );
    assert_eq!(
        Decimal256::try_from(SignedDecimal::percent(50)),
        Ok(Decimal256::percent(50))
    );
}

#[test]
fn decimal256_from_signed_decimal256_works() {
    assert_eq!(
        Decimal256::try_from(SignedDecimal256::new(Int256::MAX)),
        Ok(Decimal256::new(Uint256::MAX / Uint256::new(2)))
    );
    assert_eq!(
        Decimal256::try_from(SignedDecimal256::new(Int256::MIN)),
        Err(Decimal256RangeExceeded)
    );
    assert_eq!(
        Decimal256::try_from(SignedDecimal256::zero()),
        Ok(Decimal256::zero())
    );
    assert_eq!(
        Decimal256::try_from(SignedDecimal256::one()),
        Ok(Decimal256::one())
    );
    assert_eq!(
        Decimal256::try_from(SignedDecimal256::percent(50)),
        Ok(Decimal256::percent(50))
    );
}

#[test]
fn decimal256_from_atomics_works() {
    let one = Decimal256::one();
    let two = one + one;

    assert_eq!(Decimal256::from_atomics(1u128, 0).unwrap(), one);
    assert_eq!(Decimal256::from_atomics(10u128, 1).unwrap(), one);
    assert_eq!(Decimal256::from_atomics(100u128, 2).unwrap(), one);
    assert_eq!(Decimal256::from_atomics(1000u128, 3).unwrap(), one);
    assert_eq!(
        Decimal256::from_atomics(1000000000000000000u128, 18).unwrap(),
        one
    );
    assert_eq!(
        Decimal256::from_atomics(10000000000000000000u128, 19).unwrap(),
        one
    );
    assert_eq!(
        Decimal256::from_atomics(100000000000000000000u128, 20).unwrap(),
        one
    );

    assert_eq!(Decimal256::from_atomics(2u128, 0).unwrap(), two);
    assert_eq!(Decimal256::from_atomics(20u128, 1).unwrap(), two);
    assert_eq!(Decimal256::from_atomics(200u128, 2).unwrap(), two);
    assert_eq!(Decimal256::from_atomics(2000u128, 3).unwrap(), two);
    assert_eq!(
        Decimal256::from_atomics(2000000000000000000u128, 18).unwrap(),
        two
    );
    assert_eq!(
        Decimal256::from_atomics(20000000000000000000u128, 19).unwrap(),
        two
    );
    assert_eq!(
        Decimal256::from_atomics(200000000000000000000u128, 20).unwrap(),
        two
    );

    // Cuts decimal digits (20 provided but only 18 can be stored)
    assert_eq!(
        Decimal256::from_atomics(4321u128, 20).unwrap(),
        Decimal256::from_str("0.000000000000000043").unwrap()
    );
    assert_eq!(
        Decimal256::from_atomics(6789u128, 20).unwrap(),
        Decimal256::from_str("0.000000000000000067").unwrap()
    );
    assert_eq!(
        Decimal256::from_atomics(u128::MAX, 38).unwrap(),
        Decimal256::from_str("3.402823669209384634").unwrap()
    );
    assert_eq!(
        Decimal256::from_atomics(u128::MAX, 39).unwrap(),
        Decimal256::from_str("0.340282366920938463").unwrap()
    );
    assert_eq!(
        Decimal256::from_atomics(u128::MAX, 45).unwrap(),
        Decimal256::from_str("0.000000340282366920").unwrap()
    );
    assert_eq!(
        Decimal256::from_atomics(u128::MAX, 51).unwrap(),
        Decimal256::from_str("0.000000000000340282").unwrap()
    );
    assert_eq!(
        Decimal256::from_atomics(u128::MAX, 56).unwrap(),
        Decimal256::from_str("0.000000000000000003").unwrap()
    );
    assert_eq!(
        Decimal256::from_atomics(u128::MAX, 57).unwrap(),
        Decimal256::from_str("0.000000000000000000").unwrap()
    );
    assert_eq!(
        Decimal256::from_atomics(u128::MAX, u32::MAX).unwrap(),
        Decimal256::from_str("0.000000000000000000").unwrap()
    );

    // Can be used with max value
    let max = Decimal256::MAX;
    assert_eq!(
        Decimal256::from_atomics(max.atomics(), max.decimal_places()).unwrap(),
        max
    );

    // Overflow is only possible with digits < 18
    let result = Decimal256::from_atomics(Uint256::MAX, 17);
    assert_eq!(result.unwrap_err(), Decimal256RangeExceeded);
}

#[test]
fn decimal256_from_ratio_works() {
    // 1.0
    assert_eq!(Decimal256::from_ratio(1u128, 1u128), Decimal256::one());
    assert_eq!(Decimal256::from_ratio(53u128, 53u128), Decimal256::one());
    assert_eq!(Decimal256::from_ratio(125u128, 125u128), Decimal256::one());

    // 1.5
    assert_eq!(
        Decimal256::from_ratio(3u128, 2u128),
        Decimal256::percent(150)
    );
    assert_eq!(
        Decimal256::from_ratio(150u128, 100u128),
        Decimal256::percent(150)
    );
    assert_eq!(
        Decimal256::from_ratio(333u128, 222u128),
        Decimal256::percent(150)
    );

    // 0.125
    assert_eq!(
        Decimal256::from_ratio(1u64, 8u64),
        Decimal256::permille(125)
    );
    assert_eq!(
        Decimal256::from_ratio(125u64, 1000u64),
        Decimal256::permille(125)
    );

    // 1/3 (result floored)
    assert_eq!(
        Decimal256::from_ratio(1u64, 3u64),
        Decimal256::new(Uint256::from_str("333333333333333333").unwrap())
    );

    // 2/3 (result floored)
    assert_eq!(
        Decimal256::from_ratio(2u64, 3u64),
        Decimal256::new(Uint256::from_str("666666666666666666").unwrap())
    );

    // large inputs
    assert_eq!(Decimal256::from_ratio(0u128, u128::MAX), Decimal256::zero());
    assert_eq!(
        Decimal256::from_ratio(u128::MAX, u128::MAX),
        Decimal256::one()
    );
    // 340282366920938463463 is the largest integer <= Decimal256::MAX
    assert_eq!(
        Decimal256::from_ratio(340282366920938463463u128, 1u128),
        Decimal256::from_str("340282366920938463463").unwrap()
    );
}

#[test]
#[should_panic(expected = "Denominator must not be zero")]
fn decimal256_from_ratio_panics_for_zero_denominator() {
    Decimal256::from_ratio(1u128, 0u128);
}

#[test]
#[should_panic(expected = "Multiplication overflow")]
fn decimal256_from_ratio_panics_for_mul_overflow() {
    Decimal256::from_ratio(Uint256::MAX, 1u128);
}

#[test]
fn decimal256_checked_from_ratio_does_not_panic() {
    assert_eq!(
        Decimal256::checked_from_ratio(1u128, 0u128),
        Err(CheckedFromRatioError::DivideByZero)
    );

    assert_eq!(
        Decimal256::checked_from_ratio(Uint256::MAX, 1u128),
        Err(CheckedFromRatioError::Overflow)
    );
}

#[test]
fn decimal256_implements_fraction() {
    let fraction = Decimal256::from_str("1234.567").unwrap();
    assert_eq!(
        fraction.numerator(),
        Uint256::from_str("1234567000000000000000").unwrap()
    );
    assert_eq!(
        fraction.denominator(),
        Uint256::from_str("1000000000000000000").unwrap()
    );
}

#[test]
fn decimal256_implements_from_decimal() {
    let a = Decimal::from_str("123.456").unwrap();
    let b = Decimal256::from(a);
    assert_eq!(b.to_string(), "123.456");

    let a = Decimal::from_str("0").unwrap();
    let b = Decimal256::from(a);
    assert_eq!(b.to_string(), "0");

    let a = Decimal::MAX;
    let b = Decimal256::from(a);
    assert_eq!(b.to_string(), "340282366920938463463.374607431768211455");
}

#[test]
fn decimal256_from_str_works() {
    // Integers
    assert_eq!(Decimal256::from_str("0").unwrap(), Decimal256::percent(0));
    assert_eq!(Decimal256::from_str("1").unwrap(), Decimal256::percent(100));
    assert_eq!(Decimal256::from_str("5").unwrap(), Decimal256::percent(500));
    assert_eq!(
        Decimal256::from_str("42").unwrap(),
        Decimal256::percent(4200)
    );
    assert_eq!(Decimal256::from_str("000").unwrap(), Decimal256::percent(0));
    assert_eq!(
        Decimal256::from_str("001").unwrap(),
        Decimal256::percent(100)
    );
    assert_eq!(
        Decimal256::from_str("005").unwrap(),
        Decimal256::percent(500)
    );
    assert_eq!(
        Decimal256::from_str("0042").unwrap(),
        Decimal256::percent(4200)
    );

    // Decimals
    assert_eq!(
        Decimal256::from_str("1.0").unwrap(),
        Decimal256::percent(100)
    );
    assert_eq!(
        Decimal256::from_str("1.5").unwrap(),
        Decimal256::percent(150)
    );
    assert_eq!(
        Decimal256::from_str("0.5").unwrap(),
        Decimal256::percent(50)
    );
    assert_eq!(
        Decimal256::from_str("0.123").unwrap(),
        Decimal256::permille(123)
    );

    assert_eq!(
        Decimal256::from_str("40.00").unwrap(),
        Decimal256::percent(4000)
    );
    assert_eq!(
        Decimal256::from_str("04.00").unwrap(),
        Decimal256::percent(400)
    );
    assert_eq!(
        Decimal256::from_str("00.40").unwrap(),
        Decimal256::percent(40)
    );
    assert_eq!(
        Decimal256::from_str("00.04").unwrap(),
        Decimal256::percent(4)
    );

    // Can handle 18 fractional digits
    assert_eq!(
        Decimal256::from_str("7.123456789012345678").unwrap(),
        Decimal256::new(Uint256::from(7123456789012345678u128))
    );
    assert_eq!(
        Decimal256::from_str("7.999999999999999999").unwrap(),
        Decimal256::new(Uint256::from(7999999999999999999u128))
    );

    // Works for documented max value
    assert_eq!(
        Decimal256::from_str(
            "115792089237316195423570985008687907853269984665640564039457.584007913129639935"
        )
        .unwrap(),
        Decimal256::MAX
    );
}

#[test]
fn decimal256_from_str_errors_for_broken_whole_part() {
    assert!(Decimal256::from_str("").is_err());
    assert!(Decimal256::from_str(" ").is_err());
    assert!(Decimal256::from_str("-1").is_err());
}

#[test]
fn decimal256_from_str_errors_for_broken_fractional_part() {
    assert!(Decimal256::from_str("1.").is_err());
    assert!(Decimal256::from_str("1. ").is_err());
    assert!(Decimal256::from_str("1.e").is_err());
    assert!(Decimal256::from_str("1.2e3").is_err());
}

#[test]
fn decimal256_from_str_errors_for_more_than_36_fractional_digits() {
    assert!(Decimal256::from_str("7.1234567890123456789")
        .unwrap_err()
        .to_string()
        .ends_with("Cannot parse more than 18 fractional digits"));

    // No special rules for trailing zeros. This could be changed but adds gas cost for the happy path.
    assert!(Decimal256::from_str("7.1230000000000000000")
        .unwrap_err()
        .to_string()
        .ends_with("Cannot parse more than 18 fractional digits"));
}

#[test]
fn decimal256_from_str_errors_for_invalid_number_of_dots() {
    assert!(Decimal256::from_str("1.2.3").is_err());
    assert!(Decimal256::from_str("1.2.3.4").is_err());
}

#[test]
fn decimal256_from_str_errors_for_more_than_max_value() {
    // Integer
    assert!(
        Decimal256::from_str("115792089237316195423570985008687907853269984665640564039458")
            .is_err()
    );

    // Decimal
    assert!(
        Decimal256::from_str("115792089237316195423570985008687907853269984665640564039458.0")
            .is_err()
    );
    assert!(Decimal256::from_str(
        "115792089237316195423570985008687907853269984665640564039457.584007913129639936",
    )
    .is_err());
}

#[test]
fn decimal256_atomics_works() {
    let zero = Decimal256::zero();
    let one = Decimal256::one();
    let half = Decimal256::percent(50);
    let two = Decimal256::percent(200);
    let max = Decimal256::MAX;

    assert_eq!(zero.atomics(), Uint256::from(0u128));
    assert_eq!(one.atomics(), Uint256::from(1000000000000000000u128));
    assert_eq!(half.atomics(), Uint256::from(500000000000000000u128));
    assert_eq!(two.atomics(), Uint256::from(2000000000000000000u128));
    assert_eq!(max.atomics(), Uint256::MAX);
}

#[test]
fn decimal256_decimal_places_works() {
    let zero = Decimal256::zero();
    let one = Decimal256::one();
    let half = Decimal256::percent(50);
    let two = Decimal256::percent(200);
    let max = Decimal256::MAX;

    assert_eq!(zero.decimal_places(), 18);
    assert_eq!(one.decimal_places(), 18);
    assert_eq!(half.decimal_places(), 18);
    assert_eq!(two.decimal_places(), 18);
    assert_eq!(max.decimal_places(), 18);
}

#[test]
fn decimal256_is_zero_works() {
    assert!(Decimal256::zero().is_zero());
    assert!(Decimal256::percent(0).is_zero());
    assert!(Decimal256::permille(0).is_zero());

    assert!(!Decimal256::one().is_zero());
    assert!(!Decimal256::percent(123).is_zero());
    assert!(!Decimal256::permille(1234).is_zero());
}

#[test]
fn decimal256_inv_works() {
    // d = 0
    assert_eq!(Decimal256::zero().inv(), None);

    // d == 1
    assert_eq!(Decimal256::one().inv(), Some(Decimal256::one()));

    // d > 1 exact
    assert_eq!(
        Decimal256::from_str("2").unwrap().inv(),
        Some(Decimal256::from_str("0.5").unwrap())
    );
    assert_eq!(
        Decimal256::from_str("20").unwrap().inv(),
        Some(Decimal256::from_str("0.05").unwrap())
    );
    assert_eq!(
        Decimal256::from_str("200").unwrap().inv(),
        Some(Decimal256::from_str("0.005").unwrap())
    );
    assert_eq!(
        Decimal256::from_str("2000").unwrap().inv(),
        Some(Decimal256::from_str("0.0005").unwrap())
    );

    // d > 1 rounded
    assert_eq!(
        Decimal256::from_str("3").unwrap().inv(),
        Some(Decimal256::from_str("0.333333333333333333").unwrap())
    );
    assert_eq!(
        Decimal256::from_str("6").unwrap().inv(),
        Some(Decimal256::from_str("0.166666666666666666").unwrap())
    );

    // d < 1 exact
    assert_eq!(
        Decimal256::from_str("0.5").unwrap().inv(),
        Some(Decimal256::from_str("2").unwrap())
    );
    assert_eq!(
        Decimal256::from_str("0.05").unwrap().inv(),
        Some(Decimal256::from_str("20").unwrap())
    );
    assert_eq!(
        Decimal256::from_str("0.005").unwrap().inv(),
        Some(Decimal256::from_str("200").unwrap())
    );
    assert_eq!(
        Decimal256::from_str("0.0005").unwrap().inv(),
        Some(Decimal256::from_str("2000").unwrap())
    );
}

#[test]
#[allow(clippy::op_ref)]
fn decimal256_add_works() {
    let value = Decimal256::one() + Decimal256::percent(50); // 1.5
    assert_eq!(
        value,
        Decimal256::new(DECIMAL_FRACTIONAL * Uint256::from(3u8) / Uint256::from(2u8))
    );

    assert_eq!(
        Decimal256::percent(5) + Decimal256::percent(4),
        Decimal256::percent(9)
    );
    assert_eq!(
        Decimal256::percent(5) + Decimal256::zero(),
        Decimal256::percent(5)
    );
    assert_eq!(Decimal256::zero() + Decimal256::zero(), Decimal256::zero());

    // works for refs
    let a = Decimal256::percent(15);
    let b = Decimal256::percent(25);
    let expected = Decimal256::percent(40);
    assert_eq!(a + b, expected);
    assert_eq!(&a + b, expected);
    assert_eq!(a + &b, expected);
    assert_eq!(&a + &b, expected);
}

#[test]
#[should_panic(expected = "attempt to add with overflow")]
fn decimal256_add_overflow_panics() {
    let _value = Decimal256::MAX + Decimal256::percent(50);
}

#[test]
fn decimal256_add_assign_works() {
    let mut a = Decimal256::percent(30);
    a += Decimal256::percent(20);
    assert_eq!(a, Decimal256::percent(50));

    // works for refs
    let mut a = Decimal256::percent(15);
    let b = Decimal256::percent(3);
    let expected = Decimal256::percent(18);
    a += &b;
    assert_eq!(a, expected);
}

#[test]
#[allow(clippy::op_ref)]
fn decimal256_sub_works() {
    let value = Decimal256::one() - Decimal256::percent(50); // 0.5
    assert_eq!(
        value,
        Decimal256::new(DECIMAL_FRACTIONAL / Uint256::from(2u8))
    );

    assert_eq!(
        Decimal256::percent(9) - Decimal256::percent(4),
        Decimal256::percent(5)
    );
    assert_eq!(
        Decimal256::percent(16) - Decimal256::zero(),
        Decimal256::percent(16)
    );
    assert_eq!(
        Decimal256::percent(16) - Decimal256::percent(16),
        Decimal256::zero()
    );
    assert_eq!(Decimal256::zero() - Decimal256::zero(), Decimal256::zero());

    // works for refs
    let a = Decimal256::percent(13);
    let b = Decimal256::percent(6);
    let expected = Decimal256::percent(7);
    assert_eq!(a - b, expected);
    assert_eq!(&a - b, expected);
    assert_eq!(a - &b, expected);
    assert_eq!(&a - &b, expected);
}

#[test]
#[should_panic(expected = "attempt to subtract with overflow")]
fn decimal256_sub_overflow_panics() {
    let _value = Decimal256::zero() - Decimal256::percent(50);
}

#[test]
fn decimal256_sub_assign_works() {
    let mut a = Decimal256::percent(20);
    a -= Decimal256::percent(2);
    assert_eq!(a, Decimal256::percent(18));

    // works for refs
    let mut a = Decimal256::percent(33);
    let b = Decimal256::percent(13);
    let expected = Decimal256::percent(20);
    a -= &b;
    assert_eq!(a, expected);
}

#[test]
#[allow(clippy::op_ref)]
fn decimal256_implements_mul() {
    let one = Decimal256::one();
    let two = one + one;
    let half = Decimal256::percent(50);

    // 1*x and x*1
    assert_eq!(one * Decimal256::percent(0), Decimal256::percent(0));
    assert_eq!(one * Decimal256::percent(1), Decimal256::percent(1));
    assert_eq!(one * Decimal256::percent(10), Decimal256::percent(10));
    assert_eq!(one * Decimal256::percent(100), Decimal256::percent(100));
    assert_eq!(one * Decimal256::percent(1000), Decimal256::percent(1000));
    assert_eq!(one * Decimal256::MAX, Decimal256::MAX);
    assert_eq!(Decimal256::percent(0) * one, Decimal256::percent(0));
    assert_eq!(Decimal256::percent(1) * one, Decimal256::percent(1));
    assert_eq!(Decimal256::percent(10) * one, Decimal256::percent(10));
    assert_eq!(Decimal256::percent(100) * one, Decimal256::percent(100));
    assert_eq!(Decimal256::percent(1000) * one, Decimal256::percent(1000));
    assert_eq!(Decimal256::MAX * one, Decimal256::MAX);

    // double
    assert_eq!(two * Decimal256::percent(0), Decimal256::percent(0));
    assert_eq!(two * Decimal256::percent(1), Decimal256::percent(2));
    assert_eq!(two * Decimal256::percent(10), Decimal256::percent(20));
    assert_eq!(two * Decimal256::percent(100), Decimal256::percent(200));
    assert_eq!(two * Decimal256::percent(1000), Decimal256::percent(2000));
    assert_eq!(Decimal256::percent(0) * two, Decimal256::percent(0));
    assert_eq!(Decimal256::percent(1) * two, Decimal256::percent(2));
    assert_eq!(Decimal256::percent(10) * two, Decimal256::percent(20));
    assert_eq!(Decimal256::percent(100) * two, Decimal256::percent(200));
    assert_eq!(Decimal256::percent(1000) * two, Decimal256::percent(2000));

    // half
    assert_eq!(half * Decimal256::percent(0), Decimal256::percent(0));
    assert_eq!(half * Decimal256::percent(1), Decimal256::permille(5));
    assert_eq!(half * Decimal256::percent(10), Decimal256::percent(5));
    assert_eq!(half * Decimal256::percent(100), Decimal256::percent(50));
    assert_eq!(half * Decimal256::percent(1000), Decimal256::percent(500));
    assert_eq!(Decimal256::percent(0) * half, Decimal256::percent(0));
    assert_eq!(Decimal256::percent(1) * half, Decimal256::permille(5));
    assert_eq!(Decimal256::percent(10) * half, Decimal256::percent(5));
    assert_eq!(Decimal256::percent(100) * half, Decimal256::percent(50));
    assert_eq!(Decimal256::percent(1000) * half, Decimal256::percent(500));

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
    let max = Decimal256::MAX;
    assert_eq!(
        max * dec("1.0"),
        dec("115792089237316195423570985008687907853269984665640564039457.584007913129639935")
    );
    assert_eq!(
        max * dec("0.1"),
        dec("11579208923731619542357098500868790785326998466564056403945.758400791312963993")
    );
    assert_eq!(
        max * dec("0.01"),
        dec("1157920892373161954235709850086879078532699846656405640394.575840079131296399")
    );
    assert_eq!(
        max * dec("0.001"),
        dec("115792089237316195423570985008687907853269984665640564039.457584007913129639")
    );
    assert_eq!(
        max * dec("0.000001"),
        dec("115792089237316195423570985008687907853269984665640564.039457584007913129")
    );
    assert_eq!(
        max * dec("0.000000001"),
        dec("115792089237316195423570985008687907853269984665640.564039457584007913")
    );
    assert_eq!(
        max * dec("0.000000000001"),
        dec("115792089237316195423570985008687907853269984665.640564039457584007")
    );
    assert_eq!(
        max * dec("0.000000000000001"),
        dec("115792089237316195423570985008687907853269984.665640564039457584")
    );
    assert_eq!(
        max * dec("0.000000000000000001"),
        dec("115792089237316195423570985008687907853269.984665640564039457")
    );

    // works for refs
    let a = Decimal256::percent(20);
    let b = Decimal256::percent(30);
    let expected = Decimal256::percent(6);
    assert_eq!(a * b, expected);
    assert_eq!(&a * b, expected);
    assert_eq!(a * &b, expected);
    assert_eq!(&a * &b, expected);
}

#[test]
fn decimal256_mul_assign_works() {
    let mut a = Decimal256::percent(15);
    a *= Decimal256::percent(60);
    assert_eq!(a, Decimal256::percent(9));

    // works for refs
    let mut a = Decimal256::percent(50);
    let b = Decimal256::percent(20);
    a *= &b;
    assert_eq!(a, Decimal256::percent(10));
}

#[test]
#[should_panic(expected = "attempt to multiply with overflow")]
fn decimal256_mul_overflow_panics() {
    let _value = Decimal256::MAX * Decimal256::percent(101);
}

#[test]
fn decimal256_checked_mul() {
    let test_data = [
        (Decimal256::zero(), Decimal256::zero()),
        (Decimal256::zero(), Decimal256::one()),
        (Decimal256::one(), Decimal256::zero()),
        (Decimal256::percent(10), Decimal256::zero()),
        (Decimal256::percent(10), Decimal256::percent(5)),
        (Decimal256::MAX, Decimal256::one()),
        (
            Decimal256::MAX / Uint256::from_uint128(2u128.into()),
            Decimal256::percent(200),
        ),
        (Decimal256::permille(6), Decimal256::permille(13)),
    ];

    // The regular core::ops::Mul is our source of truth for these tests.
    for (x, y) in test_data.into_iter() {
        assert_eq!(x * y, x.checked_mul(y).unwrap());
    }
}

#[test]
fn decimal256_checked_mul_overflow() {
    assert_eq!(
        Decimal256::MAX.checked_mul(Decimal256::percent(200)),
        Err(OverflowError::new(OverflowOperation::Mul))
    );
}

#[test]
#[allow(clippy::op_ref)]
fn decimal256_implements_div() {
    let one = Decimal256::one();
    let two = one + one;
    let half = Decimal256::percent(50);

    // 1/x and x/1
    assert_eq!(one / Decimal256::percent(1), Decimal256::percent(10_000));
    assert_eq!(one / Decimal256::percent(10), Decimal256::percent(1_000));
    assert_eq!(one / Decimal256::percent(100), Decimal256::percent(100));
    assert_eq!(one / Decimal256::percent(1000), Decimal256::percent(10));
    assert_eq!(Decimal256::percent(0) / one, Decimal256::percent(0));
    assert_eq!(Decimal256::percent(1) / one, Decimal256::percent(1));
    assert_eq!(Decimal256::percent(10) / one, Decimal256::percent(10));
    assert_eq!(Decimal256::percent(100) / one, Decimal256::percent(100));
    assert_eq!(Decimal256::percent(1000) / one, Decimal256::percent(1000));

    // double
    assert_eq!(two / Decimal256::percent(1), Decimal256::percent(20_000));
    assert_eq!(two / Decimal256::percent(10), Decimal256::percent(2_000));
    assert_eq!(two / Decimal256::percent(100), Decimal256::percent(200));
    assert_eq!(two / Decimal256::percent(1000), Decimal256::percent(20));
    assert_eq!(Decimal256::percent(0) / two, Decimal256::percent(0));
    assert_eq!(Decimal256::percent(1) / two, dec("0.005"));
    assert_eq!(Decimal256::percent(10) / two, Decimal256::percent(5));
    assert_eq!(Decimal256::percent(100) / two, Decimal256::percent(50));
    assert_eq!(Decimal256::percent(1000) / two, Decimal256::percent(500));

    // half
    assert_eq!(half / Decimal256::percent(1), Decimal256::percent(5_000));
    assert_eq!(half / Decimal256::percent(10), Decimal256::percent(500));
    assert_eq!(half / Decimal256::percent(100), Decimal256::percent(50));
    assert_eq!(half / Decimal256::percent(1000), Decimal256::percent(5));
    assert_eq!(Decimal256::percent(0) / half, Decimal256::percent(0));
    assert_eq!(Decimal256::percent(1) / half, Decimal256::percent(2));
    assert_eq!(Decimal256::percent(10) / half, Decimal256::percent(20));
    assert_eq!(Decimal256::percent(100) / half, Decimal256::percent(200));
    assert_eq!(Decimal256::percent(1000) / half, Decimal256::percent(2000));

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
        Decimal256::percent(15) / Decimal256::percent(60),
        Decimal256::percent(25)
    );

    // works for refs
    let a = Decimal256::percent(100);
    let b = Decimal256::percent(20);
    let expected = Decimal256::percent(500);
    assert_eq!(a / b, expected);
    assert_eq!(&a / b, expected);
    assert_eq!(a / &b, expected);
    assert_eq!(&a / &b, expected);
}

#[test]
fn decimal256_div_assign_works() {
    let mut a = Decimal256::percent(15);
    a /= Decimal256::percent(20);
    assert_eq!(a, Decimal256::percent(75));

    // works for refs
    let mut a = Decimal256::percent(50);
    let b = Decimal256::percent(20);
    a /= &b;
    assert_eq!(a, Decimal256::percent(250));
}

#[test]
#[should_panic(expected = "Division failed - multiplication overflow")]
fn decimal256_div_overflow_panics() {
    let _value = Decimal256::MAX / Decimal256::percent(10);
}

#[test]
#[should_panic(expected = "Division failed - denominator must not be zero")]
fn decimal256_div_by_zero_panics() {
    let _value = Decimal256::one() / Decimal256::zero();
}

#[test]
fn decimal256_uint128_division() {
    // a/b
    let left = Decimal256::percent(150); // 1.5
    let right = Uint256::from(3u128);
    assert_eq!(left / right, Decimal256::percent(50));

    // 0/a
    let left = Decimal256::zero();
    let right = Uint256::from(300u128);
    assert_eq!(left / right, Decimal256::zero());
}

#[test]
#[should_panic(expected = "attempt to divide by zero")]
fn decimal256_uint128_divide_by_zero() {
    let left = Decimal256::percent(150); // 1.5
    let right = Uint256::from(0u128);
    let _result = left / right;
}

#[test]
fn decimal256_uint128_div_assign() {
    // a/b
    let mut dec = Decimal256::percent(150); // 1.5
    dec /= Uint256::from(3u128);
    assert_eq!(dec, Decimal256::percent(50));

    // 0/a
    let mut dec = Decimal256::zero();
    dec /= Uint256::from(300u128);
    assert_eq!(dec, Decimal256::zero());
}

#[test]
#[should_panic(expected = "attempt to divide by zero")]
fn decimal256_uint128_div_assign_by_zero() {
    // a/0
    let mut dec = Decimal256::percent(50);
    dec /= Uint256::from(0u128);
}

#[test]
fn decimal256_uint128_sqrt() {
    assert_eq!(Decimal256::percent(0).sqrt(), Decimal256::percent(0));
    assert_eq!(Decimal256::percent(900).sqrt(), Decimal256::percent(300));
    assert!(Decimal256::percent(316) < Decimal256::percent(1000).sqrt());
    assert!(Decimal256::percent(1000).sqrt() < Decimal256::percent(317));
}

/// sqrt(2) is an irrational number, i.e. all 36 decimal places should be used.
#[test]
fn decimal256_uint128_sqrt_is_precise() {
    assert_eq!(
        Decimal256::from_str("2").unwrap().sqrt(),
        Decimal256::from_str("1.414213562373095048").unwrap() // https://www.wolframalpha.com/input/?i=sqrt%282%29
    );
}

#[test]
fn decimal256_uint128_sqrt_does_not_overflow() {
    assert_eq!(
        Decimal256::from_str("40000000000000000000000000000000000000000000000000000000000")
            .unwrap()
            .sqrt(),
        Decimal256::from_str("200000000000000000000000000000").unwrap()
    );
}

#[test]
fn decimal256_uint128_sqrt_intermediate_precision_used() {
    assert_eq!(
        Decimal256::from_str("40000000000000000000000000000000000000000000000001")
            .unwrap()
            .sqrt(),
        // The last few digits (39110) are truncated below due to the algorithm
        // we use. Larger numbers will cause less precision.
        // https://www.wolframalpha.com/input/?i=sqrt%2840000000000000000000000000000000000000000000000001%29
        Decimal256::from_str("6324555320336758663997787.088865437067400000").unwrap()
    );
}

#[test]
fn decimal256_checked_pow() {
    for exp in 0..10 {
        assert_eq!(
            Decimal256::one().checked_pow(exp).unwrap(),
            Decimal256::one()
        );
    }

    // This case is mathematically undefined, but we ensure consistency with Rust standard types
    // https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=20df6716048e77087acd40194b233494
    assert_eq!(
        Decimal256::zero().checked_pow(0).unwrap(),
        Decimal256::one()
    );

    for exp in 1..10 {
        assert_eq!(
            Decimal256::zero().checked_pow(exp).unwrap(),
            Decimal256::zero()
        );
    }

    for num in &[
        Decimal256::percent(50),
        Decimal256::percent(99),
        Decimal256::percent(200),
    ] {
        assert_eq!(num.checked_pow(0).unwrap(), Decimal256::one())
    }

    assert_eq!(
        Decimal256::percent(20).checked_pow(2).unwrap(),
        Decimal256::percent(4)
    );

    assert_eq!(
        Decimal256::percent(20).checked_pow(3).unwrap(),
        Decimal256::permille(8)
    );

    assert_eq!(
        Decimal256::percent(200).checked_pow(4).unwrap(),
        Decimal256::percent(1600)
    );

    assert_eq!(
        Decimal256::percent(200).checked_pow(4).unwrap(),
        Decimal256::percent(1600)
    );

    assert_eq!(
        Decimal256::percent(700).checked_pow(5).unwrap(),
        Decimal256::percent(1680700)
    );

    assert_eq!(
        Decimal256::percent(700).checked_pow(8).unwrap(),
        Decimal256::percent(576480100)
    );

    assert_eq!(
        Decimal256::percent(700).checked_pow(10).unwrap(),
        Decimal256::percent(28247524900)
    );

    assert_eq!(
        Decimal256::percent(120).checked_pow(123).unwrap(),
        Decimal256::new(5486473221892422150877397607u128.into())
    );

    assert_eq!(
        Decimal256::percent(10).checked_pow(2).unwrap(),
        Decimal256::new(10000000000000000u128.into())
    );

    assert_eq!(
        Decimal256::percent(10).checked_pow(18).unwrap(),
        Decimal256::new(1u128.into())
    );
}

#[test]
fn decimal256_checked_pow_overflow() {
    assert_eq!(
        Decimal256::MAX.checked_pow(2),
        Err(OverflowError::new(OverflowOperation::Pow))
    );
    assert_eq!(
        Decimal256::MAX.checked_pow(3),
        Err(OverflowError::new(OverflowOperation::Pow))
    );
    assert_eq!(
        Decimal256::new(DECIMAL_FRACTIONAL * Uint256::new(1_000_000_000)).checked_pow(15),
        Err(OverflowError::new(OverflowOperation::Pow))
    );
}

#[test]
fn decimal256_to_string() {
    // Integers
    assert_eq!(Decimal256::zero().to_string(), "0");
    assert_eq!(Decimal256::one().to_string(), "1");
    assert_eq!(Decimal256::percent(500).to_string(), "5");

    // Decimals
    assert_eq!(Decimal256::percent(125).to_string(), "1.25");
    assert_eq!(Decimal256::percent(42638).to_string(), "426.38");
    assert_eq!(Decimal256::percent(3).to_string(), "0.03");
    assert_eq!(Decimal256::permille(987).to_string(), "0.987");

    assert_eq!(
        Decimal256::new(Uint256::from(1u128)).to_string(),
        "0.000000000000000001"
    );
    assert_eq!(
        Decimal256::new(Uint256::from(10u128)).to_string(),
        "0.00000000000000001"
    );
    assert_eq!(
        Decimal256::new(Uint256::from(100u128)).to_string(),
        "0.0000000000000001"
    );
    assert_eq!(
        Decimal256::new(Uint256::from(1000u128)).to_string(),
        "0.000000000000001"
    );
    assert_eq!(
        Decimal256::new(Uint256::from(10000u128)).to_string(),
        "0.00000000000001"
    );
    assert_eq!(
        Decimal256::new(Uint256::from(100000u128)).to_string(),
        "0.0000000000001"
    );
    assert_eq!(
        Decimal256::new(Uint256::from(1000000u128)).to_string(),
        "0.000000000001"
    );
    assert_eq!(
        Decimal256::new(Uint256::from(10000000u128)).to_string(),
        "0.00000000001"
    );
    assert_eq!(
        Decimal256::new(Uint256::from(100000000u128)).to_string(),
        "0.0000000001"
    );
    assert_eq!(
        Decimal256::new(Uint256::from(1000000000u128)).to_string(),
        "0.000000001"
    );
    assert_eq!(
        Decimal256::new(Uint256::from(10000000000u128)).to_string(),
        "0.00000001"
    );
    assert_eq!(
        Decimal256::new(Uint256::from(100000000000u128)).to_string(),
        "0.0000001"
    );
    assert_eq!(
        Decimal256::new(Uint256::from(10000000000000u128)).to_string(),
        "0.00001"
    );
    assert_eq!(
        Decimal256::new(Uint256::from(100000000000000u128)).to_string(),
        "0.0001"
    );
    assert_eq!(
        Decimal256::new(Uint256::from(1000000000000000u128)).to_string(),
        "0.001"
    );
    assert_eq!(
        Decimal256::new(Uint256::from(10000000000000000u128)).to_string(),
        "0.01"
    );
    assert_eq!(
        Decimal256::new(Uint256::from(100000000000000000u128)).to_string(),
        "0.1"
    );
}

#[test]
fn decimal256_iter_sum() {
    let items = vec![
        Decimal256::zero(),
        Decimal256::from_str("2").unwrap(),
        Decimal256::from_str("2").unwrap(),
    ];
    assert_eq!(
        items.iter().sum::<Decimal256>(),
        Decimal256::from_str("4").unwrap()
    );
    assert_eq!(
        items.into_iter().sum::<Decimal256>(),
        Decimal256::from_str("4").unwrap()
    );

    let empty: Vec<Decimal256> = vec![];
    assert_eq!(Decimal256::zero(), empty.iter().sum::<Decimal256>());
}

#[test]
fn decimal256_serialize() {
    assert_eq!(serde_json::to_vec(&Decimal256::zero()).unwrap(), br#""0""#);
    assert_eq!(serde_json::to_vec(&Decimal256::one()).unwrap(), br#""1""#);
    assert_eq!(
        serde_json::to_vec(&Decimal256::percent(8)).unwrap(),
        br#""0.08""#
    );
    assert_eq!(
        serde_json::to_vec(&Decimal256::percent(87)).unwrap(),
        br#""0.87""#
    );
    assert_eq!(
        serde_json::to_vec(&Decimal256::percent(876)).unwrap(),
        br#""8.76""#
    );
    assert_eq!(
        serde_json::to_vec(&Decimal256::percent(8765)).unwrap(),
        br#""87.65""#
    );
}

#[test]
fn decimal256_deserialize() {
    assert_eq!(
        serde_json::from_slice::<Decimal256>(br#""0""#).unwrap(),
        Decimal256::zero()
    );
    assert_eq!(
        serde_json::from_slice::<Decimal256>(br#""1""#).unwrap(),
        Decimal256::one()
    );
    assert_eq!(
        serde_json::from_slice::<Decimal256>(br#""000""#).unwrap(),
        Decimal256::zero()
    );
    assert_eq!(
        serde_json::from_slice::<Decimal256>(br#""001""#).unwrap(),
        Decimal256::one()
    );

    assert_eq!(
        serde_json::from_slice::<Decimal256>(br#""0.08""#).unwrap(),
        Decimal256::percent(8)
    );
    assert_eq!(
        serde_json::from_slice::<Decimal256>(br#""0.87""#).unwrap(),
        Decimal256::percent(87)
    );
    assert_eq!(
        serde_json::from_slice::<Decimal256>(br#""8.76""#).unwrap(),
        Decimal256::percent(876)
    );
    assert_eq!(
        serde_json::from_slice::<Decimal256>(br#""87.65""#).unwrap(),
        Decimal256::percent(8765)
    );
}

#[test]
fn decimal256_abs_diff_works() {
    let a = Decimal256::percent(285);
    let b = Decimal256::percent(200);
    let expected = Decimal256::percent(85);
    assert_eq!(a.abs_diff(b), expected);
    assert_eq!(b.abs_diff(a), expected);
}

#[test]
#[allow(clippy::op_ref)]
fn decimal256_rem_works() {
    // 4.02 % 1.11 = 0.69
    assert_eq!(
        Decimal256::percent(402) % Decimal256::percent(111),
        Decimal256::percent(69)
    );

    // 15.25 % 4 = 3.25
    assert_eq!(
        Decimal256::percent(1525) % Decimal256::percent(400),
        Decimal256::percent(325)
    );

    let a = Decimal256::percent(318);
    let b = Decimal256::percent(317);
    let expected = Decimal256::percent(1);
    assert_eq!(a % b, expected);
    assert_eq!(a % &b, expected);
    assert_eq!(&a % b, expected);
    assert_eq!(&a % &b, expected);
}

#[test]
fn decimal_rem_assign_works() {
    let mut a = Decimal256::percent(17673);
    a %= Decimal256::percent(2362);
    assert_eq!(a, Decimal256::percent(1139)); // 176.73 % 23.62 = 11.39

    let mut a = Decimal256::percent(4262);
    let b = Decimal256::percent(1270);
    a %= &b;
    assert_eq!(a, Decimal256::percent(452)); // 42.62 % 12.7 = 4.52
}

#[test]
#[should_panic(expected = "divisor of zero")]
fn decimal256_rem_panics_for_zero() {
    let _ = Decimal256::percent(777) % Decimal256::zero();
}

#[test]
fn decimal256_checked_methods() {
    // checked add
    assert_eq!(
        Decimal256::percent(402)
            .checked_add(Decimal256::percent(111))
            .unwrap(),
        Decimal256::percent(513)
    );
    assert!(matches!(
        Decimal256::MAX.checked_add(Decimal256::percent(1)),
        Err(OverflowError { .. })
    ));

    // checked sub
    assert_eq!(
        Decimal256::percent(1111)
            .checked_sub(Decimal256::percent(111))
            .unwrap(),
        Decimal256::percent(1000)
    );
    assert!(matches!(
        Decimal256::zero().checked_sub(Decimal256::percent(1)),
        Err(OverflowError { .. })
    ));

    // checked div
    assert_eq!(
        Decimal256::percent(30)
            .checked_div(Decimal256::percent(200))
            .unwrap(),
        Decimal256::percent(15)
    );
    assert_eq!(
        Decimal256::percent(88)
            .checked_div(Decimal256::percent(20))
            .unwrap(),
        Decimal256::percent(440)
    );
    assert!(matches!(
        Decimal256::MAX.checked_div(Decimal256::zero()),
        Err(CheckedFromRatioError::DivideByZero)
    ));
    assert!(matches!(
        Decimal256::MAX.checked_div(Decimal256::percent(1)),
        Err(CheckedFromRatioError::Overflow)
    ));

    // checked rem
    assert_eq!(
        Decimal256::percent(402)
            .checked_rem(Decimal256::percent(111))
            .unwrap(),
        Decimal256::percent(69)
    );
    assert_eq!(
        Decimal256::percent(1525)
            .checked_rem(Decimal256::percent(400))
            .unwrap(),
        Decimal256::percent(325)
    );
    assert!(matches!(
        Decimal256::MAX.checked_rem(Decimal256::zero()),
        Err(DivideByZeroError { .. })
    ));
}

#[test]
fn decimal256_pow_works() {
    assert_eq!(Decimal256::percent(200).pow(2), Decimal256::percent(400));
    assert_eq!(
        Decimal256::percent(200).pow(10),
        Decimal256::percent(102400)
    );
}

#[test]
#[should_panic(expected = "Multiplication overflow")]
fn decimal256_pow_overflow_panics() {
    _ = Decimal256::MAX.pow(2u32);
}

#[test]
fn decimal256_saturating_works() {
    assert_eq!(
        Decimal256::percent(200).saturating_add(Decimal256::percent(200)),
        Decimal256::percent(400)
    );
    assert_eq!(
        Decimal256::MAX.saturating_add(Decimal256::percent(200)),
        Decimal256::MAX
    );
    assert_eq!(
        Decimal256::percent(200).saturating_sub(Decimal256::percent(100)),
        Decimal256::percent(100)
    );
    assert_eq!(
        Decimal256::zero().saturating_sub(Decimal256::percent(200)),
        Decimal256::zero()
    );
    assert_eq!(
        Decimal256::percent(200).saturating_mul(Decimal256::percent(50)),
        Decimal256::percent(100)
    );
    assert_eq!(
        Decimal256::MAX.saturating_mul(Decimal256::percent(200)),
        Decimal256::MAX
    );
    assert_eq!(
        Decimal256::percent(400).saturating_pow(2u32),
        Decimal256::percent(1600)
    );
    assert_eq!(Decimal256::MAX.saturating_pow(2u32), Decimal256::MAX);
}

#[test]
fn decimal256_rounding() {
    assert_eq!(Decimal256::one().floor(), Decimal256::one());
    assert_eq!(Decimal256::percent(150).floor(), Decimal256::one());
    assert_eq!(Decimal256::percent(199).floor(), Decimal256::one());
    assert_eq!(Decimal256::percent(200).floor(), Decimal256::percent(200));
    assert_eq!(Decimal256::percent(99).floor(), Decimal256::zero());

    assert_eq!(Decimal256::one().ceil(), Decimal256::one());
    assert_eq!(Decimal256::percent(150).ceil(), Decimal256::percent(200));
    assert_eq!(Decimal256::percent(199).ceil(), Decimal256::percent(200));
    assert_eq!(Decimal256::percent(99).ceil(), Decimal256::one());
    assert_eq!(
        Decimal256::new(Uint256::from(1u128)).ceil(),
        Decimal256::one()
    );
}

#[test]
#[should_panic(expected = "attempt to ceil with overflow")]
fn decimal256_ceil_panics() {
    let _ = Decimal256::MAX.ceil();
}

#[test]
fn decimal256_checked_ceil() {
    assert_eq!(
        Decimal256::percent(199).checked_ceil(),
        Ok(Decimal256::percent(200))
    );
    assert_eq!(Decimal256::MAX.checked_ceil(), Err(RoundUpOverflowError));
}

#[test]
fn decimal256_to_uint_floor_works() {
    let d = Decimal256::from_str("12.000000000000000001").unwrap();
    assert_eq!(d.to_uint_floor(), Uint256::new(12));
    let d = Decimal256::from_str("12.345").unwrap();
    assert_eq!(d.to_uint_floor(), Uint256::new(12));
    let d = Decimal256::from_str("12.999").unwrap();
    assert_eq!(d.to_uint_floor(), Uint256::new(12));
    let d = Decimal256::from_str("0.98451384").unwrap();
    assert_eq!(d.to_uint_floor(), Uint256::new(0));

    let d = Decimal256::from_str("75.0").unwrap();
    assert_eq!(d.to_uint_floor(), Uint256::new(75));
    let d = Decimal256::from_str("0.0").unwrap();
    assert_eq!(d.to_uint_floor(), Uint256::new(0));

    let d = Decimal256::MAX;
    assert_eq!(
        d.to_uint_floor(),
        Uint256::from_str("115792089237316195423570985008687907853269984665640564039457").unwrap()
    );

    // Does the same as the old workaround `Uint256::one() * my_decimal`.
    // This block can be deleted as part of https://github.com/CosmWasm/cosmwasm/issues/1485.
    let tests = vec![
        (
            Decimal256::from_str("12.345").unwrap(),
            Uint256::from(12u128),
        ),
        (
            Decimal256::from_str("0.98451384").unwrap(),
            Uint256::from(0u128),
        ),
        (
            Decimal256::from_str("178.0").unwrap(),
            Uint256::from(178u128),
        ),
        (Decimal256::MIN, Uint256::from(0u128)),
        (Decimal256::MAX, Uint256::MAX / DECIMAL_FRACTIONAL),
    ];
    for (my_decimal, expected) in tests.into_iter() {
        assert_eq!(my_decimal.to_uint_floor(), expected);
    }
}

#[test]
fn decimal256_to_uint_ceil_works() {
    let d = Decimal256::from_str("12.000000000000000001").unwrap();
    assert_eq!(d.to_uint_ceil(), Uint256::new(13));
    let d = Decimal256::from_str("12.345").unwrap();
    assert_eq!(d.to_uint_ceil(), Uint256::new(13));
    let d = Decimal256::from_str("12.999").unwrap();
    assert_eq!(d.to_uint_ceil(), Uint256::new(13));

    let d = Decimal256::from_str("75.0").unwrap();
    assert_eq!(d.to_uint_ceil(), Uint256::new(75));
    let d = Decimal256::from_str("0.0").unwrap();
    assert_eq!(d.to_uint_ceil(), Uint256::new(0));

    let d = Decimal256::MAX;
    assert_eq!(
        d.to_uint_ceil(),
        Uint256::from_str("115792089237316195423570985008687907853269984665640564039458").unwrap()
    );
}

#[test]
fn decimal256_partial_eq() {
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
fn decimal256_implements_debug() {
    let decimal = Decimal256::from_str("123.45").unwrap();
    assert_eq!(format!("{decimal:?}"), "Decimal256(123.45)");

    let test_cases = ["5", "5.01", "42", "0", "2"];
    for s in test_cases {
        let decimal256 = Decimal256::from_str(s).unwrap();
        let expected = format!("Decimal256({s})");
        assert_eq!(format!("{decimal256:?}"), expected);
    }
}

#[test]
fn serialize_decimal256_to_string() {
    let d = Decimal256::from_str("123.45").unwrap();
    let json_string = serde_json::to_string(&d).unwrap();
    assert_eq!(r#""123.45""#, json_string);
}

#[test]
fn deserialize_decimal256_from_string() {
    let json_string = r#""123.45""#;
    let d: Decimal256 = serde_json::from_str(json_string).unwrap();
    assert_eq!(d, Decimal256::from_str("123.45").unwrap());
}

#[test]
fn deserialize256_invalid_decimal() {
    let json_string = r#""123,45""#;
    let result: Result<Decimal256, _> = serde_json::from_str(json_string);
    let err = result.unwrap_err();
    assert!(err.to_string().contains("invalid digit"));
}

#[test]
fn deserialize_wrong_type_triggers_expectation() {
    let json_decimal = "123.45";
    let err = serde_json::from_str::<Decimal256>(json_decimal).unwrap_err();
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
        Decimal256::from_str("123.456").unwrap()
    )
    .unwrap_err();

    write!(
        &mut FailingWriter::new(When::OnDecimal),
        "{}",
        Decimal256::from_str("123.456").unwrap()
    )
    .unwrap_err();

    write!(
        &mut FailingWriter::new(When::AfterDecimal),
        "{}",
        Decimal256::from_str("123.456").unwrap()
    )
    .unwrap_err();
}
