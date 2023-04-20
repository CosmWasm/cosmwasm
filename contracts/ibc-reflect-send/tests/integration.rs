//! This integration test tries to run and call the generated wasm.
//! It depends on a Wasm build being available, which you can create with `cargo wasm`.
//! Then running `cargo integration-test` will validate we can properly call into that generated Wasm.
//!
//! You can easily convert unit tests to integration tests.
//! 1. First copy them over verbatum,
//! 2. Then change
//!      let mut deps = mock_dependencies(20, &[]);
//!    to
//!      let mut deps = mock_instance(WASM, &[]);
//! 3. If you access raw storage, where ever you see something like:
//!      deps.storage.get(CONFIG_KEY).expect("no data stored");
//!    replace it with:
//!      deps.with_storage(|store| {
//!          let data = store.get(CONFIG_KEY).expect("no data stored");
//!          //...
//!      });
//! 4. Anywhere you see query(&deps, ...) you must replace it with query(&mut deps, ...)

use cosmwasm_std::testing::{
    mock_ibc_channel_connect_ack, mock_ibc_channel_open_init, mock_ibc_channel_open_try,
    mock_ibc_packet_ack,
};
use cosmwasm_std::{
    attr, coin, coins, BankMsg, CosmosMsg, Empty, IbcAcknowledgement, IbcBasicResponse, IbcMsg,
    IbcOrder, Response,
};
use cosmwasm_vm::testing::{
    execute, ibc_channel_connect, ibc_channel_open, ibc_packet_ack, instantiate, mock_env,
    mock_info, mock_instance, query, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_vm::{from_slice, Instance};

use ibc_reflect_send::ibc::IBC_APP_VERSION;
use ibc_reflect_send::ibc_msg::{AcknowledgementMsg, PacketMsg, WhoAmIResponse};
use ibc_reflect_send::msg::{AccountResponse, AdminResponse, ExecuteMsg, InstantiateMsg, QueryMsg};

// This line will test the output of cargo wasm
static WASM: &[u8] =
    include_bytes!("../target/wasm32-unknown-unknown/release/ibc_reflect_send.wasm");

const CREATOR: &str = "creator";

const DESERIALIZATION_LIMIT: usize = 20_000;

fn setup() -> Instance<MockApi, MockStorage, MockQuerier> {
    let mut deps = mock_instance(WASM, &[]);
    let msg = InstantiateMsg {};
    let info = mock_info(CREATOR, &[]);
    let res: Response = instantiate(&mut deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    deps
}

// connect will run through the entire handshake to set up a proper connect and
// save the account (tested in detail in `proper_handshake_flow`)
fn connect(deps: &mut Instance<MockApi, MockStorage, MockQuerier>, channel_id: &str) {
    // open packet has no counterparty version, connect does
    let handshake_open = mock_ibc_channel_open_init(channel_id, IbcOrder::Ordered, IBC_APP_VERSION);
    // first we try to open with a valid handshake
    ibc_channel_open(deps, mock_env(), handshake_open).unwrap();

    // then we connect (with counter-party version set)
    let handshake_connect =
        mock_ibc_channel_connect_ack(channel_id, IbcOrder::Ordered, IBC_APP_VERSION);
    let res: IbcBasicResponse = ibc_channel_connect(deps, mock_env(), handshake_connect).unwrap();

    // this should send a WhoAmI request, which is received some blocks later
    assert_eq!(1, res.messages.len());
    match &res.messages[0].msg {
        CosmosMsg::Ibc(IbcMsg::SendPacket {
            channel_id: packet_channel,
            ..
        }) => assert_eq!(packet_channel.as_str(), channel_id),
        o => panic!("Unexpected message: {:?}", o),
    };
}

fn who_am_i_response(
    deps: &mut Instance<MockApi, MockStorage, MockQuerier>,
    channel_id: &str,
    account: impl Into<String>,
) {
    let packet = PacketMsg::WhoAmI {};
    let response = AcknowledgementMsg::Ok(WhoAmIResponse {
        account: account.into(),
    });
    let ack = IbcAcknowledgement::encode_json(&response).unwrap();
    let msg = mock_ibc_packet_ack(channel_id, &packet, ack).unwrap();
    let res: IbcBasicResponse = ibc_packet_ack(deps, mock_env(), msg).unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn instantiate_works() {
    let mut deps = setup();
    let r = query(&mut deps, mock_env(), QueryMsg::Admin {}).unwrap();
    let admin: AdminResponse = from_slice(&r, DESERIALIZATION_LIMIT).unwrap();
    assert_eq!(CREATOR, admin.admin.as_str());
}

#[test]
fn enforce_version_in_handshake() {
    let mut deps = setup();

    let wrong_order = mock_ibc_channel_open_try("channel-12", IbcOrder::Unordered, IBC_APP_VERSION);
    ibc_channel_open(&mut deps, mock_env(), wrong_order).unwrap_err();

    let wrong_version = mock_ibc_channel_open_try("channel-12", IbcOrder::Ordered, "reflect");
    ibc_channel_open(&mut deps, mock_env(), wrong_version).unwrap_err();

    let valid_handshake =
        mock_ibc_channel_open_try("channel-12", IbcOrder::Ordered, IBC_APP_VERSION);
    ibc_channel_open(&mut deps, mock_env(), valid_handshake).unwrap();
}

fn get_account(
    deps: &mut Instance<MockApi, MockStorage, MockQuerier>,
    channel_id: &str,
) -> AccountResponse {
    let msg = QueryMsg::Account {
        channel_id: channel_id.into(),
    };
    let r = query(deps, mock_env(), msg).unwrap();
    from_slice(&r, DESERIALIZATION_LIMIT).unwrap()
}

#[test]
fn proper_handshake_flow() {
    // setup and connect handshake
    let mut deps = setup();
    let channel_id = "channel-1234";
    connect(&mut deps, channel_id);

    // check for empty account
    let acct = get_account(&mut deps, channel_id);
    assert!(acct.remote_addr.is_none());
    assert!(acct.remote_balance.is_empty());
    assert_eq!(0, acct.last_update_time.nanos());

    // now get feedback from WhoAmI packet
    let remote_addr = "account-789";
    who_am_i_response(&mut deps, channel_id, remote_addr);

    // account should be set up
    let acct = get_account(&mut deps, channel_id);
    assert_eq!(acct.remote_addr.unwrap(), remote_addr);
    assert!(acct.remote_balance.is_empty());
    assert_eq!(0, acct.last_update_time.nanos());
}

#[test]
fn dispatch_message_send_and_ack() {
    let channel_id = "channel-1234";
    let remote_addr = "account-789";

    // init contract
    let mut deps = setup();
    // channel handshake
    connect(&mut deps, channel_id);
    // get feedback from WhoAmI packet
    who_am_i_response(&mut deps, channel_id, remote_addr);

    // try to dispatch a message
    let msgs_to_dispatch = vec![BankMsg::Send {
        to_address: "my-friend".into(),
        amount: coins(123456789, "uatom"),
    }
    .into()];
    let execute_msg = ExecuteMsg::SendMsgs {
        channel_id: channel_id.into(),
        msgs: msgs_to_dispatch,
    };
    let info = mock_info(CREATOR, &[]);
    let mut res: Response = execute(&mut deps, mock_env(), info, execute_msg).unwrap();
    assert_eq!(1, res.messages.len());
    let msg = match res.messages.swap_remove(0).msg {
        CosmosMsg::Ibc(IbcMsg::SendPacket {
            channel_id, data, ..
        }) => {
            let ack = IbcAcknowledgement::encode_json(&AcknowledgementMsg::Ok(())).unwrap();
            let mut msg = mock_ibc_packet_ack(&channel_id, &1, ack).unwrap();
            msg.original_packet.data = data;
            msg
        }
        o => panic!("Unexpected message: {:?}", o),
    };
    let res: IbcBasicResponse = ibc_packet_ack(&mut deps, mock_env(), msg).unwrap();
    // no actions expected, but let's check the events to see it was dispatched properly
    assert_eq!(0, res.messages.len());
    assert_eq!(vec![attr("action", "acknowledge_dispatch")], res.attributes)
}

#[test]
fn send_remote_funds() {
    let reflect_channel_id = "channel-1234";
    let remote_addr = "account-789";
    let transfer_channel_id = "transfer-2";

    // init contract
    let mut deps = setup();
    // channel handshake
    connect(&mut deps, reflect_channel_id);
    // get feedback from WhoAmI packet
    who_am_i_response(&mut deps, reflect_channel_id, remote_addr);

    // let's try to send funds to a channel that doesn't exist
    let msg = ExecuteMsg::SendFunds {
        reflect_channel_id: "random-channel".into(),
        transfer_channel_id: transfer_channel_id.into(),
    };
    let info = mock_info(CREATOR, &coins(12344, "utrgd"));
    execute::<_, _, _, _, Empty>(&mut deps, mock_env(), info, msg).unwrap_err();

    // let's try with no sent funds in the message
    let msg = ExecuteMsg::SendFunds {
        reflect_channel_id: reflect_channel_id.into(),
        transfer_channel_id: transfer_channel_id.into(),
    };
    let info = mock_info(CREATOR, &[]);
    execute::<_, _, _, _, Empty>(&mut deps, mock_env(), info, msg).unwrap_err();

    // 3rd times the charm
    let msg = ExecuteMsg::SendFunds {
        reflect_channel_id: reflect_channel_id.into(),
        transfer_channel_id: transfer_channel_id.into(),
    };
    let info = mock_info(CREATOR, &coins(12344, "utrgd"));
    let res: Response = execute(&mut deps, mock_env(), info, msg).unwrap();
    assert_eq!(1, res.messages.len());
    match &res.messages[0].msg {
        CosmosMsg::Ibc(IbcMsg::Transfer {
            channel_id,
            to_address,
            amount,
            timeout,
            memo,
        }) => {
            assert_eq!(transfer_channel_id, channel_id.as_str());
            assert_eq!(remote_addr, to_address.as_str());
            assert_eq!(&coin(12344, "utrgd"), amount);
            assert!(timeout.block().is_none());
            assert!(timeout.timestamp().is_some());
        }
        o => panic!("unexpected message: {:?}", o),
    }
}
