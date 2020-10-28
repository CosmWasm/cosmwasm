pub mod singlepass;

use crate::context::Env;
use crate::traits::{Querier, Storage};

#[derive(Debug)]
pub struct InsufficientGasLeft;

/// Decreases gas left by the given amount.
/// If the amount exceeds the available gas, the remaining gas is set to 0 and
/// an InsufficientGasLeft error is returned.
pub fn decrease_gas_left<S: Storage, Q: Querier>(
    env: &mut Env<S, Q>,
    amount: u64,
) -> Result<(), InsufficientGasLeft> {
    let remaining = get_gas_left(env);
    if amount > remaining {
        set_gas_left(env, 0);
        Err(InsufficientGasLeft)
    } else {
        set_gas_left(env, remaining - amount);
        Ok(())
    }
}

pub use singlepass::{backend, compile, get_gas_left, set_gas_left};

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::{MockQuerier, MockStorage};
    use std::ptr::NonNull;
    use wabt::wat2wasm;
    use wasmer::{imports, Instance as WasmerInstance};

    type MS = MockStorage;
    type MQ = MockQuerier;
    const GAS_LIMIT: u64 = 5_000_000;

    fn instantiate(code: &[u8]) -> (Env<MS, MQ>, Box<WasmerInstance>) {
        let mut env = Env::new(GAS_LIMIT);
        let module = compile(code).unwrap();
        let import_obj = imports! { "env" => {}, };
        let instance = Box::from(WasmerInstance::new(&module, &import_obj).unwrap());

        let instance_ptr = NonNull::from(instance.as_ref());
        env.set_wasmer_instance(Some(instance_ptr));

        (env, instance)
    }

    #[test]
    fn decrease_gas_left_works() {
        let wasm = wat2wasm("(module)").unwrap();
        let (mut env, _) = instantiate(&wasm);

        let before = get_gas_left(&env);
        decrease_gas_left(&mut env, 32).unwrap();
        let after = get_gas_left(&env);
        assert_eq!(after, before - 32);
    }

    #[test]
    fn decrease_gas_left_can_consume_all_gas() {
        let wasm = wat2wasm("(module)").unwrap();
        let (mut env, _) = instantiate(&wasm);

        let before = get_gas_left(&env);
        decrease_gas_left(&mut env, before).unwrap();
        let after = get_gas_left(&env);
        assert_eq!(after, 0);
    }

    #[test]
    fn decrease_gas_left_errors_for_amount_greater_than_remaining() {
        let wasm = wat2wasm("(module)").unwrap();
        let (mut env, _) = instantiate(&wasm);

        let before = get_gas_left(&env);
        let result = decrease_gas_left(&mut env, before + 1);
        match result.unwrap_err() {
            InsufficientGasLeft => {}
        }
        let after = get_gas_left(&env);
        assert_eq!(after, 0);
    }
}
