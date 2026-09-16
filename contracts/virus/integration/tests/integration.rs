use cosmwasm_vm::testing::mock_instance;

static WASM: &[u8] = include_bytes!("../../target/wasm32-unknown-unknown/release/virus.wasm");

#[test]
fn validation_succeeds() {
    mock_instance(WASM, &[]);
}
