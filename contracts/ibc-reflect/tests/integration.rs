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

use cosmwasm_std::{
    coins, BankMsg, ContractResult, CosmosMsg, HandleResponse, HumanAddr, IbcBasicResponse,
    IbcChannel, IbcEndpoint, IbcOrder, IbcPacket, IbcReceiveResponse, IbcTimeoutHeight,
    InitResponse, WasmMsg,
};
use cosmwasm_vm::testing::{
    handle, ibc_channel_connect, ibc_channel_open, ibc_packet_receive, init, mock_env, mock_info,
    mock_instance, query, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_vm::{from_slice, to_vec, Instance};

use ibc_reflect::contract::IBC_VERSION;
use ibc_reflect::msg::{
    AccountInfo, AccountResponse, AcknowledgementMsg, HandleMsg, InitMsg, ListAccountsResponse,
    PacketMsg, QueryMsg, ReflectHandleMsg, ReflectInitMsg,
};
use serde::Serialize;

// This line will test the output of cargo wasm
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/ibc_reflect.wasm");

const CREATOR: &str = "creator";
// code id of the reflect contract
const REFLECT_ID: u64 = 101;
// address of first reflect contract instance that we created
const REFLECT_ADDR: &str = "reflect-acct-1";

fn setup() -> Instance<MockApi, MockStorage, MockQuerier> {
    let mut deps = mock_instance(WASM, &[]);
    let msg = InitMsg {
        reflect_code_id: REFLECT_ID,
    };
    let info = mock_info(CREATOR, &[]);
    let res: InitResponse = init(&mut deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    deps
}

// this provides a minimal channel skeleton, which can be modified to set certain fields
// TODO: something similar should be a helper in ibc.rs
fn mock_channel(order: IbcOrder, version: &str) -> IbcChannel {
    IbcChannel {
        endpoint: IbcEndpoint {
            port_id: "my_port".to_string(),
            channel_id: "channel-5".to_string(),
        },
        counterparty_endpoint: IbcEndpoint {
            port_id: "their_port".to_string(),
            channel_id: "channel-7".to_string(),
        },
        order,
        version: version.to_string(),
        /// CounterpartyVersion can be None when not known this context, yet
        counterparty_version: None,
        /// The connection upon which this channel was created. If this is a multi-hop
        /// channel, we only expose the first hop.
        connection_id: "connection-2".to_string(),
    }
}

// this provides a minimal packet around custom data, which can be modified to set certain fields
// TODO: something similar should be a helper in ibc.rs
fn mock_ibc_packet<T: Serialize>(channel_id: &str, data: &T) -> IbcPacket {
    IbcPacket {
        data: to_vec(data).unwrap().into(),
        src: IbcEndpoint {
            port_id: "their-port".to_string(),
            channel_id: "channel-1234".to_string(),
        },
        dest: IbcEndpoint {
            port_id: "our-port".to_string(),
            channel_id: channel_id.into(),
        },
        sequence: 27,
        timeout_height: IbcTimeoutHeight {
            revision_number: 1,
            timeout_height: 12345678,
        },
        timeout_timestamp: 0,
        version: 1,
    }
}

// connect will run through the entire handshake to set up a proper connect and
// save the account (tested in detail in `proper_handshake_flow`)
fn connect<T: Into<HumanAddr>>(
    deps: &mut Instance<MockApi, MockStorage, MockQuerier>,
    channel_id: &str,
    account: T,
) {
    let account = account.into();
    // first we try to open with a valid handshake
    let mut valid_handshake = mock_channel(IbcOrder::Ordered, IBC_VERSION);
    valid_handshake.endpoint.channel_id = channel_id.into();
    ibc_channel_open(deps, mock_env(), valid_handshake.clone()).unwrap();

    // then we connect (with counter-party version set)
    valid_handshake.counterparty_version = Some(IBC_VERSION.to_string());
    let _: IbcBasicResponse = ibc_channel_connect(deps, mock_env(), valid_handshake).unwrap();

    // which creates a reflect account. here we get the callback
    let handle_msg = HandleMsg::InitCallback {
        id: channel_id.into(),
        contract_addr: account.clone(),
    };
    let info = mock_info(account, &[]);
    let _: HandleResponse = handle(deps, mock_env(), info, handle_msg).unwrap();
}

#[test]
fn init_works() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InitMsg {
        reflect_code_id: 17,
    };
    let info = mock_info("creator", &[]);
    let res: ContractResult<InitResponse> = init(&mut deps, mock_env(), info, msg);
    let msgs = res.unwrap().messages;
    assert_eq!(0, msgs.len());
}

#[test]
fn enforce_version_in_handshake() {
    let mut deps = setup();

    let wrong_order = mock_channel(IbcOrder::Unordered, IBC_VERSION);
    ibc_channel_open(&mut deps, mock_env(), wrong_order).unwrap_err();

    let wrong_version = mock_channel(IbcOrder::Ordered, "reflect");
    ibc_channel_open(&mut deps, mock_env(), wrong_version).unwrap_err();

    let valid_handshake = mock_channel(IbcOrder::Ordered, IBC_VERSION);
    ibc_channel_open(&mut deps, mock_env(), valid_handshake).unwrap();
}

#[test]
fn proper_handshake_flow() {
    let mut deps = setup();

    // first we try to open with a valid handshake
    let mut valid_handshake = mock_channel(IbcOrder::Ordered, IBC_VERSION);
    ibc_channel_open(&mut deps, mock_env(), valid_handshake.clone()).unwrap();

    // then we connect (with counter-party version set)
    valid_handshake.counterparty_version = Some(IBC_VERSION.to_string());
    let res: IbcBasicResponse =
        ibc_channel_connect(&mut deps, mock_env(), valid_handshake.clone()).unwrap();
    // and set up a reflect account
    assert_eq!(1, res.messages.len());
    let our_channel = valid_handshake.endpoint.channel_id.clone();
    if let CosmosMsg::Wasm(WasmMsg::Instantiate {
        code_id,
        msg,
        send,
        label,
    }) = &res.messages[0]
    {
        assert_eq!(&REFLECT_ID, code_id);
        assert_eq!(0, send.len());
        assert!(label.as_ref().unwrap().contains(&our_channel));
        // parse the message - should callback with proper channel_id
        let rmsg: ReflectInitMsg = from_slice(&msg).unwrap();
        assert_eq!(rmsg.callback_id, Some(our_channel.clone()));
    } else {
        panic!("invalid return message: {:?}", res.messages[0]);
    }

    // no accounts set yet
    let raw = query(&mut deps, mock_env(), QueryMsg::ListAccounts {}).unwrap();
    let res: ListAccountsResponse = from_slice(&raw).unwrap();
    assert_eq!(0, res.accounts.len());

    // we get the callback from reflect
    let handle_msg = HandleMsg::InitCallback {
        id: our_channel.clone(),
        contract_addr: REFLECT_ADDR.into(),
    };
    let info = mock_info(REFLECT_ADDR, &[]);
    let res: HandleResponse = handle(&mut deps, mock_env(), info, handle_msg).unwrap();
    assert_eq!(0, res.messages.len());

    // ensure this is now registered
    let raw = query(&mut deps, mock_env(), QueryMsg::ListAccounts {}).unwrap();
    let res: ListAccountsResponse = from_slice(&raw).unwrap();
    assert_eq!(1, res.accounts.len());
    assert_eq!(
        &res.accounts[0],
        &AccountInfo {
            account: REFLECT_ADDR.into(),
            channel_id: our_channel.clone(),
        }
    );

    // and the account query also works
    let raw = query(
        &mut deps,
        mock_env(),
        QueryMsg::Account {
            channel_id: our_channel.clone(),
        },
    )
    .unwrap();
    let res: AccountResponse = from_slice(&raw).unwrap();
    assert_eq!(res.account.unwrap(), HumanAddr::from(REFLECT_ADDR));
}

#[test]
fn handle_packet() {
    let mut deps = setup();

    let channel_id: &str = "channel-123";
    let account: &str = "acct-123";

    // receive a packet for an unregistered channel returns app-level error (not Result::Err)
    let ibc_msg = PacketMsg {
        msgs: vec![BankMsg::Send {
            to_address: "my-friend".into(),
            amount: coins(123456789, "uatom"),
        }
        .into()],
    };
    let packet = mock_ibc_packet(channel_id, &ibc_msg);
    let res: IbcReceiveResponse =
        ibc_packet_receive(&mut deps, mock_env(), packet.clone()).unwrap();
    // we didn't dispatch anything
    assert_eq!(0, res.messages.len());
    // acknowledgement is an error
    let ack: AcknowledgementMsg = from_slice(&res.acknowledgement).unwrap();
    assert_eq!(ack.unwrap_err(), "invalid packet");

    // register the channel
    connect(&mut deps, channel_id, account);

    // receive a packet for an unregistered channel returns app-level error (not Result::Err)
    let packet = mock_ibc_packet(channel_id, &ibc_msg);
    let res: IbcReceiveResponse =
        ibc_packet_receive(&mut deps, mock_env(), packet.clone()).unwrap();
    println!(
        "{}",
        String::from_utf8(res.acknowledgement.0.clone()).unwrap()
    );

    // assert app-level success
    let ack: AcknowledgementMsg = from_slice(&res.acknowledgement).unwrap();
    ack.unwrap();

    // and we dispatch the BankMsg
    assert_eq!(1, res.messages.len());
    // parse the output, ensuring it matches
    if let CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr,
        msg,
        send,
    }) = &res.messages[0]
    {
        assert_eq!(account, contract_addr.as_str());
        assert_eq!(0, send.len());
        // parse the message - should callback with proper channel_id
        let rmsg: ReflectHandleMsg = from_slice(&msg).unwrap();
        assert_eq!(
            rmsg,
            ReflectHandleMsg::ReflectMsg {
                msgs: ibc_msg.msgs.clone()
            }
        );
    } else {
        panic!("invalid return message: {:?}", res.messages[0]);
    }

    // invalid packet format on registered channel also returns app-level error
    let bad_data = InitMsg {
        reflect_code_id: 12345,
    };
    let packet = mock_ibc_packet(channel_id, &bad_data);
    let res: IbcReceiveResponse =
        ibc_packet_receive(&mut deps, mock_env(), packet.clone()).unwrap();
    // we didn't dispatch anything
    assert_eq!(0, res.messages.len());
    // acknowledgement is an error
    let ack: AcknowledgementMsg = from_slice(&res.acknowledgement).unwrap();
    assert_eq!(ack.unwrap_err(), "invalid packet");
}
