pub mod cranelift;
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

#[cfg(feature = "default-cranelift")]
pub use cranelift::{backend, compile, get_gas_left, set_gas_left};

#[cfg(feature = "default-singlepass")]
pub use singlepass::{backend, compile, get_gas_left, set_gas_left};

#[cfg(test)]
#[cfg(feature = "default-singlepass")]
mod test {
    use super::*;
    use wabt::wat2wasm;
    use wasmer::{imports, Instance as WasmerInstance};

    fn instantiate(code: &[u8]) -> WasmerInstance {
        let module = compile(code).unwrap();
        let import_obj = imports! { "env" => {}, };
        WasmerInstance::new(&module, &import_obj).unwrap()
    }

    #[test]
    fn decrease_gas_left_works() {
        let wasm = wat2wasm("(module)").unwrap();
        let mut instance = instantiate(&wasm);

        let before = get_gas_left(&instance.context());
        decrease_gas_left(&mut instance.context_mut(), 32).unwrap();
        let after = get_gas_left(&instance.context());
        assert_eq!(after, before - 32);
    }

    #[test]
    fn decrease_gas_left_can_consume_all_gas() {
        let wasm = wat2wasm("(module)").unwrap();
        let mut instance = instantiate(&wasm);

        let before = get_gas_left(&instance.context());
        decrease_gas_left(&mut instance.context_mut(), before).unwrap();
        let after = get_gas_left(&instance.context());
        assert_eq!(after, 0);
    }

    #[test]
    fn decrease_gas_left_errors_for_amount_greater_than_remaining() {
        let wasm = wat2wasm("(module)").unwrap();
        let mut instance = instantiate(&wasm);

        let before = get_gas_left(&instance.context());
        let result = decrease_gas_left(&mut instance.context_mut(), before + 1);
        match result.unwrap_err() {
            InsufficientGasLeft => {}
        }
        let after = get_gas_left(&instance.context());
        assert_eq!(after, 0);
    }
}
