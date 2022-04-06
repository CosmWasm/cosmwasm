use cosmwasm_vm::testing::mock_instance;

// This line will test the output of cargo wasm
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/floaty.wasm");
// You can uncomment this line instead to test productionified build from rust-optimizer
// static WASM: &[u8] = include_bytes!("../contract.wasm");

#[test]
#[should_panic(expected = "Float operator detected")]
fn instantiate_fails() {
    let mut _deps = mock_instance(WASM, &[]);
}
