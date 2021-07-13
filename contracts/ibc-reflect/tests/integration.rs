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

use cosmwasm_std::testing::{mock_ibc_channel, mock_ibc_packet_recv};
use cosmwasm_std::{
    attr, coins, BankMsg, ContractResult, CosmosMsg, Event, IbcBasicResponse, IbcChannelConnectMsg,
    IbcChannelOpenMsg, IbcOrder, IbcReceiveResponse, Reply, Response, SubMsgExecutionResponse,
    WasmMsg,
};
use cosmwasm_vm::testing::{
    ibc_channel_connect, ibc_channel_open, ibc_packet_receive, instantiate, mock_env, mock_info,
    mock_instance, query, reply, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR,
};
use cosmwasm_vm::{from_slice, Instance};

use ibc_reflect::contract::{IBC_VERSION, RECEIVE_DISPATCH_ID};
use ibc_reflect::msg::{
    AccountInfo, AccountResponse, AcknowledgementMsg, DispatchResponse, InstantiateMsg,
    ListAccountsResponse, PacketMsg, QueryMsg, ReflectExecuteMsg,
};

// This line will test the output of cargo wasm
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/ibc_reflect.wasm");

const CREATOR: &str = "creator";
// code id of the reflect contract
const REFLECT_ID: u64 = 101;
// address of first reflect contract instance that we created
const REFLECT_ADDR: &str = "reflect-acct-1";

const DESERIALIZATION_LIMIT: usize = 20_000;

fn setup() -> Instance<MockApi, MockStorage, MockQuerier> {
    let mut deps = mock_instance(WASM, &[]);
    let msg = InstantiateMsg {
        reflect_code_id: REFLECT_ID,
    };
    let info = mock_info(CREATOR, &[]);
    let res: Response = instantiate(&mut deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    deps
}

fn fake_events(reflect_addr: &str) -> Vec<Event> {
    let event = Event {
        ty: "message".into(),
        attributes: vec![
            attr("module", "wasm"),
            attr("signer", MOCK_CONTRACT_ADDR),
            attr("code_id", "17"),
            attr("contract_address", reflect_addr),
        ],
    };
    vec![event]
}

// connect will run through the entire handshake to set up a proper connect and
// save the account (tested in detail in `proper_handshake_flow`)
fn connect<T: Into<String>>(
    deps: &mut Instance<MockApi, MockStorage, MockQuerier>,
    channel_id: &str,
    account: T,
) {
    let account: String = account.into();
    // first we try to open with a valid handshake
    let mut handshake_open =
        IbcChannelOpenMsg::new(mock_ibc_channel(channel_id, IbcOrder::Ordered, IBC_VERSION));
    handshake_open.channel.counterparty_version = None;
    ibc_channel_open(deps, mock_env(), handshake_open).unwrap();

    // then we connect (with counter-party version set)
    let handshake_connect =
        IbcChannelConnectMsg::new(mock_ibc_channel(channel_id, IbcOrder::Ordered, IBC_VERSION));
    let res: IbcBasicResponse = ibc_channel_connect(deps, mock_env(), handshake_connect).unwrap();
    assert_eq!(1, res.messages.len());
    let id = res.messages[0].id;

    // fake a reply and ensure this works
    let response = Reply {
        id,
        result: ContractResult::Ok(SubMsgExecutionResponse {
            events: fake_events(&account),
            data: None,
        }),
    };
    let _: Response = reply(deps, mock_env(), response).unwrap();
}

#[test]
fn instantiate_works() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InstantiateMsg {
        reflect_code_id: 17,
    };
    let info = mock_info("creator", &[]);
    let res: ContractResult<Response> = instantiate(&mut deps, mock_env(), info, msg);
    let msgs = res.unwrap().messages;
    assert_eq!(0, msgs.len());
}

#[test]
fn enforce_version_in_handshake() {
    let mut deps = setup();

    let wrong_order = IbcChannelOpenMsg::new(mock_ibc_channel(
        "channel-1234",
        IbcOrder::Unordered,
        IBC_VERSION,
    ));
    ibc_channel_open(&mut deps, mock_env(), wrong_order).unwrap_err();

    let wrong_version = IbcChannelOpenMsg::new(mock_ibc_channel(
        "channel-1234",
        IbcOrder::Ordered,
        "reflect",
    ));
    ibc_channel_open(&mut deps, mock_env(), wrong_version).unwrap_err();

    let valid_handshake = IbcChannelOpenMsg::new(mock_ibc_channel(
        "channel-1234",
        IbcOrder::Ordered,
        IBC_VERSION,
    ));
    ibc_channel_open(&mut deps, mock_env(), valid_handshake).unwrap();
}

#[test]
fn proper_handshake_flow() {
    let mut deps = setup();
    let channel_id = "channel-432";

    // first we try to open with a valid handshake
    let mut handshake_open =
        IbcChannelOpenMsg::new(mock_ibc_channel(channel_id, IbcOrder::Ordered, IBC_VERSION));
    handshake_open.channel.counterparty_version = None;
    ibc_channel_open(&mut deps, mock_env(), handshake_open).unwrap();

    // then we connect (with counter-party version set)
    let handshake_connect =
        IbcChannelConnectMsg::new(mock_ibc_channel(channel_id, IbcOrder::Ordered, IBC_VERSION));
    let res: IbcBasicResponse =
        ibc_channel_connect(&mut deps, mock_env(), handshake_connect).unwrap();
    // and set up a reflect account
    assert_eq!(1, res.messages.len());
    let id = res.messages[0].id;
    if let CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin,
        code_id,
        msg: _,
        funds,
        label,
    }) = &res.messages[0].msg
    {
        assert_eq!(*admin, None);
        assert_eq!(*code_id, REFLECT_ID);
        assert_eq!(funds.len(), 0);
        assert!(label.contains(channel_id));
    } else {
        panic!("invalid return message: {:?}", res.messages[0]);
    }

    // no accounts set yet
    let raw = query(&mut deps, mock_env(), QueryMsg::ListAccounts {}).unwrap();
    let res: ListAccountsResponse = from_slice(&raw, DESERIALIZATION_LIMIT).unwrap();
    assert_eq!(0, res.accounts.len());

    // we get the callback from reflect
    let response = Reply {
        id,
        result: ContractResult::Ok(SubMsgExecutionResponse {
            events: fake_events(&REFLECT_ADDR),
            data: None,
        }),
    };
    let res: Response = reply(&mut deps, mock_env(), response).unwrap();
    assert_eq!(0, res.messages.len());

    // ensure this is now registered
    let raw = query(&mut deps, mock_env(), QueryMsg::ListAccounts {}).unwrap();
    let res: ListAccountsResponse = from_slice(&raw, DESERIALIZATION_LIMIT).unwrap();
    assert_eq!(1, res.accounts.len());
    assert_eq!(
        &res.accounts[0],
        &AccountInfo {
            account: REFLECT_ADDR.into(),
            channel_id: channel_id.to_string(),
        }
    );

    // and the account query also works
    let raw = query(
        &mut deps,
        mock_env(),
        QueryMsg::Account {
            channel_id: channel_id.to_string(),
        },
    )
    .unwrap();
    let res: AccountResponse = from_slice(&raw, DESERIALIZATION_LIMIT).unwrap();
    assert_eq!(res.account.unwrap(), REFLECT_ADDR);
}

#[test]
fn handle_dispatch_packet() {
    let mut deps = setup();

    let channel_id = "channel-123";
    let account = "acct-123";

    // receive a packet for an unregistered channel returns app-level error (not Result::Err)
    let msgs_to_dispatch = vec![BankMsg::Send {
        to_address: "my-friend".into(),
        amount: coins(123456789, "uatom"),
    }
    .into()];
    let ibc_msg = PacketMsg::Dispatch {
        msgs: msgs_to_dispatch.clone(),
    };
    let msg = mock_ibc_packet_recv(channel_id, &ibc_msg).unwrap();
    let res: IbcReceiveResponse = ibc_packet_receive(&mut deps, mock_env(), msg).unwrap();
    // we didn't dispatch anything
    assert_eq!(0, res.messages.len());
    // acknowledgement is an error
    let ack: AcknowledgementMsg<DispatchResponse> =
        from_slice(&res.acknowledgement, DESERIALIZATION_LIMIT).unwrap();
    assert_eq!(
        ack.unwrap_err(),
        "invalid packet: cosmwasm_std::addresses::Addr not found"
    );

    // register the channel
    connect(&mut deps, channel_id, account);

    // receive a packet for an unregistered channel returns app-level error (not Result::Err)
    let msg = mock_ibc_packet_recv(channel_id, &ibc_msg).unwrap();
    let res: IbcReceiveResponse = ibc_packet_receive(&mut deps, mock_env(), msg).unwrap();

    // assert app-level success
    let ack: AcknowledgementMsg<DispatchResponse> =
        from_slice(&res.acknowledgement, DESERIALIZATION_LIMIT).unwrap();
    ack.unwrap();

    // and we dispatch the BankMsg
    assert_eq!(1, res.messages.len());
    assert_eq!(RECEIVE_DISPATCH_ID, res.messages[0].id);

    // parse the output, ensuring it matches
    if let CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr,
        msg,
        funds,
    }) = &res.messages[0].msg
    {
        assert_eq!(account, contract_addr.as_str());
        assert_eq!(0, funds.len());
        // parse the message - should callback with proper channel_id
        let rmsg: ReflectExecuteMsg = from_slice(&msg, DESERIALIZATION_LIMIT).unwrap();
        assert_eq!(
            rmsg,
            ReflectExecuteMsg::ReflectMsg {
                msgs: msgs_to_dispatch
            }
        );
    } else {
        panic!("invalid return message: {:?}", res.messages[0]);
    }

    // invalid packet format on registered channel also returns app-level error
    let bad_data = InstantiateMsg {
        reflect_code_id: 12345,
    };
    let msg = mock_ibc_packet_recv(channel_id, &bad_data).unwrap();
    let res: IbcReceiveResponse = ibc_packet_receive(&mut deps, mock_env(), msg).unwrap();
    // we didn't dispatch anything
    assert_eq!(0, res.messages.len());
    // acknowledgement is an error
    let ack: AcknowledgementMsg<DispatchResponse> =
        from_slice(&res.acknowledgement, DESERIALIZATION_LIMIT).unwrap();
    assert_eq!(ack.unwrap_err(), "invalid packet: Error parsing into type ibc_reflect::msg::PacketMsg: unknown variant `reflect_code_id`, expected one of `dispatch`, `who_am_i`, `balances`");
}
