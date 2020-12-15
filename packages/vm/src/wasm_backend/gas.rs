use std::convert::TryInto;
use wasmer::Instance as WasmerInstance;

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
