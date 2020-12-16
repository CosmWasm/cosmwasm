use std::convert::TryInto;
use wasmer::Instance as WasmerInstance;

// TODO: Use get_remaining_points from wasmer_middlewares (https://github.com/wasmerio/wasmer/pull/1941)
pub fn get_remaining_points(instance: &WasmerInstance) -> u64 {
    instance
        .exports
        .get_global("remaining_points")
        .expect("Can't get `remaining_points` from Instance")
        .get()
        .try_into()
        .expect("`remaining_points` from Instance has wrong type")
}

// TODO: Use set_remaining_points from wasmer_middlewares (https://github.com/wasmerio/wasmer/pull/1941)
pub fn set_remaining_points(instance: &WasmerInstance, points: u64) {
    instance
        .exports
        .get_global("remaining_points")
        .expect("Can't get `remaining_points` from Instance")
        .set(points.into())
        .expect("Can't set `remaining_points` in Instance");
}
