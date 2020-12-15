use std::convert::TryInto;
use wasmer::Instance as WasmerInstance;

use crate::backend::{Api, Querier, Storage};
use crate::environment::Environment;

/// In Wasmer, the gas limit is set on modules during compilation and is included in the cached modules.
/// This causes issues when trying to instantiate the same compiled module with a different gas limit.
/// A fix for this is proposed here: https://github.com/wasmerio/wasmer/pull/996.
///
/// To work around this limitation, we set the gas limit of all Wasmer instances to this very high value,
/// assuming users won't request more than this amount of gas. In order to set the real gas limit, we pretend
/// to consume the difference between the two in `set_gas_left` ("points used" in the metering middleware).
/// Since we observed overflow behaviour in the points used, we ensure both MAX_GAS_LIMIT and points used stay
/// far below u64::MAX.
// const MAX_GAS_LIMIT: u64 = u64::MAX / 2;

const FAKE_GAS_AVAILABLE: u64 = 1_000_000;

/// Get how many more gas units can be used in the context.
pub fn get_gas_left<A: Api, S: Storage, Q: Querier>(_env: &Environment<A, S, Q>) -> u64 {
    FAKE_GAS_AVAILABLE
}

/// A copy of https://github.com/wasmerio/wasmer/blob/873560e2033afb54e7bec123e9d2e1f6ab55fd58/lib/middlewares/src/metering.rs#L56-L66
pub fn get_gas_left_from_wasmer_instance(instance: &WasmerInstance) -> u64 {
    instance
        .exports
        .get_global("remaining_points")
        .expect("Can't get `remaining_points` from Instance")
        .get()
        .try_into()
        .expect("`remaining_points` from Instance has wrong type")
}

/// A copy of https://github.com/wasmerio/wasmer/blob/873560e2033afb54e7bec123e9d2e1f6ab55fd58/lib/middlewares/src/metering.rs#L68-L78
pub fn set_gas_left_to_wasmer_instance(instance: &WasmerInstance, new_value: u64) {
    instance
        .exports
        .get_global("remaining_points")
        .expect("Can't get `remaining_points` from Instance")
        .set(new_value.into())
        .expect("Can't set `remaining_points` in Instance");
}

// /// Set the amount of gas units that can be used in the context.
// pub fn set_gas_left(ctx: &mut Ctx, amount: u64) {
//     if amount > MAX_GAS_LIMIT {
//         panic!(
//             "Attempted to set gas limit larger than max gas limit (got: {}; maximum: {}).",
//             amount, MAX_GAS_LIMIT
//         );
//     } else {
//         let used = MAX_GAS_LIMIT - amount;
//         metering::set_points_used_ctx(ctx, used);
//     }
// }

// /// Get how many more gas units can be used in the context.
// pub fn get_gas_left(ctx: &Ctx) -> u64 {
//     let used = metering::get_points_used_ctx(ctx);
//     // when running out of gas, get_points_used can exceed MAX_GAS_LIMIT
//     MAX_GAS_LIMIT.saturating_sub(used)
// }

#[cfg(test)]
#[cfg(feature = "metering")]
mod tests {
    use super::*;
    use crate::size::Size;
    use crate::testing::{MockQuerier, MockStorage};
    use crate::wasm_backend::compile;
    use std::ptr::NonNull;
    use wasmer::{imports, Instance as WasmerInstance};

    type MS = MockStorage;
    type MQ = MockQuerier;
    const GAS_LIMIT: u64 = 5_000_000;
    const MAX_GAS_LIMIT: u64 = u64::MAX / 2;
    const TESTING_MEMORY_LIMIT: Size = Size::mebi(16);

    fn instantiate(code: &[u8]) -> (Environment<MS, MQ>, Box<WasmerInstance>) {
        let env = Environment::new(GAS_LIMIT, false);
        let module = compile(code, Some(TESTING_MEMORY_LIMIT)).unwrap();
        let import_obj = imports! { "env" => {}, };
        let instance = Box::from(WasmerInstance::new(&module, &import_obj).unwrap());

        let instance_ptr = NonNull::from(instance.as_ref());
        env.set_wasmer_instance(Some(instance_ptr));

        (env, instance)
    }

    #[test]
    fn decrease_gas_left_works() {
        let wasm = wat::parse_str("(module)").unwrap();
        let (env, _) = instantiate(&wasm);

        let before = get_gas_left(&env);
        decrease_gas_left(&env, 32).unwrap();
        let after = get_gas_left(&env);
        assert_eq!(after, before - 32);
    }

    #[test]
    fn decrease_gas_left_can_consume_all_gas() {
        let wasm = wat::parse_str("(module)").unwrap();
        let (env, _) = instantiate(&wasm);

        let before = get_gas_left(&env);
        decrease_gas_left(&env, before).unwrap();
        let after = get_gas_left(&env);
        assert_eq!(after, 0);
    }

    #[test]
    fn decrease_gas_left_errors_for_amount_greater_than_remaining() {
        let wasm = wat::parse_str("(module)").unwrap();
        let (env, _) = instantiate(&wasm);

        let before = get_gas_left(&env);
        let result = decrease_gas_left(&env, before + 1);
        match result.unwrap_err() {
            InsufficientGasLeft => {}
        }
        let after = get_gas_left(&env);
        assert_eq!(after, 0);
    }

    #[test]
    fn get_gas_left_defaults_to_constant() {
        let wasm = wat::parse_str("(module)").unwrap();
        let (env, _) = instantiate(&wasm);
        let gas_left = get_gas_left(&env);
        assert_eq!(gas_left, MAX_GAS_LIMIT);
    }

    #[test]
    fn set_gas_left_works() {
        let wasm = wat::parse_str("(module)").unwrap();
        let (env, _) = instantiate(&wasm);

        let limit = 3456789;
        set_gas_left(&env, limit);
        assert_eq!(get_gas_left(&env), limit);

        let limit = 1;
        set_gas_left(&env, limit);
        assert_eq!(get_gas_left(&env), limit);

        let limit = 0;
        set_gas_left(&env, limit);
        assert_eq!(get_gas_left(&env), limit);

        let limit = MAX_GAS_LIMIT;
        set_gas_left(&env, limit);
        assert_eq!(get_gas_left(&env), limit);
    }

    #[test]
    #[should_panic(
        expected = "Attempted to set gas limit larger than max gas limit (got: 9223372036854775808; maximum: 9223372036854775807)."
    )]
    fn set_gas_left_panic_for_values_too_large() {
        let wasm = wat::parse_str("(module)").unwrap();
        let (env, _) = instantiate(&wasm);

        let limit = MAX_GAS_LIMIT + 1;
        set_gas_left(&env, limit);
    }
}
