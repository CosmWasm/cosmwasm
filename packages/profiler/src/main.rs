mod code_blocks;
mod instrumentation;
mod measure;
mod operators;
// mod profiling;
mod utils;

use std::sync::{Arc, Mutex};

use cosmwasm_std::{coins, Response};
use cosmwasm_vm::{
    testing::{instantiate, mock_env, mock_info, query, MockApi, MockQuerier, MockStorage},
    Instance,
};

use crate::{
    code_blocks::{BlockId, BlockStore},
    instrumentation::Module,
    measure::Measurements,
};

type Env = Arc<Mutex<Measurements>>;
type MockInstance = Instance<MockApi, MockStorage, MockQuerier>;

fn main() {
    fn start_measurement(env: &Env, fn_index: u32, local_block_id: u32) {
        env.lock()
            .unwrap()
            .start_measurement(fn_index, local_block_id);
    }

    fn take_measurement(
        env: &Env,
        fn_index: u32,
        local_block_id: u32,
        block_id: impl Into<BlockId>,
    ) {
        env.lock()
            .unwrap()
            .take_measurement(fn_index, local_block_id, block_id);
    }

    let measurements = Arc::new(Mutex::new(Measurements::new()));
    let block_store = Arc::new(Mutex::new(BlockStore::new()));

    let mut instance = Module::from_path("testdata/hackatom.wasm").instrument(
        block_store.clone(),
        measurements.clone(),
        start_measurement,
        take_measurement,
    );

    eprintln!("Warm-up round: 10 executions...");
    for _ in 1..10 {
        call_things(instance.vm_instance());
    }

    {
        let mut measurements = measurements.lock().unwrap();
        measurements.clear();
    }

    eprintln!("Profiling 10 executions...");
    // This could probably be multi-threaded.
    for _ in 1..10 {
        call_things(instance.vm_instance());
    }

    let measurements = measurements.lock().unwrap();
    measurements.compile_csv(block_store, std::io::stdout());
}

// Pretty much stolen from `/contracts/hackatom/tests/integration.rs`
fn call_things(deps: &mut MockInstance) {
    use hackatom::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

    let verifier = String::from("verifies");
    let beneficiary = String::from("benefits");
    let creator = String::from("creator");
    let msg = InstantiateMsg {
        verifier,
        beneficiary,
    };
    let info = mock_info(&creator, &coins(1000, "earth"));
    let res: Response = instantiate(deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // now let's query
    let query_response = query(deps, mock_env(), QueryMsg::Verifier {}).unwrap();
    assert_eq!(query_response.as_slice(), b"{\"verifier\":\"verifies\"}");

    // bad query returns parse error (pass wrong type - this connection is not enforced)
    let qres = query(deps, mock_env(), ExecuteMsg::Release {});
    let msg = qres.unwrap_err();
    assert!(msg.contains("Error parsing"));
}
