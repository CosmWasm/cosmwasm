# IBC interfaces for CosmWasm contracts

If you import `cosmwasm-std` with the `stargate` feature flag, it will expose a
number of IBC-related functionality. This requires that the host chain is
running an IBC-enabled version of
[`x/wasmd`](https://github.com/CosmWasm/wasmd/tree/master/x/wasm), that is
`v0.16.0` or higher. You will get an error when you upload the contract if the
chain doesn't support this functionality.

## Sending Tokens via ICS20

There are two ways to use IBC. The simplest one, available to all contracts, is
simply to send tokens to another chain on a pre-established ICS20 channel. ICS20
is the protocol that is used to move fungible tokens between Cosmos blockchains.
To this end, we expose a
[new `CosmosMsg::Ibc(IbcMsg::Transfer{})` message variant](https://github.com/CosmWasm/cosmwasm/blob/v0.14.0-beta4/packages/std/src/ibc.rs#L25-L40)
that works similar to `CosmosMsg::Bank(BankMsg::Send{})`, but with a few extra
fields:

```rust
pub enum IbcMsg {
    /// Sends bank tokens owned by the contract to the given address on another chain.
    /// The channel must already be established between the ibctransfer module on this chain
    /// and a matching module on the remote chain.
    /// We cannot select the port_id, this is whatever the local chain has bound the ibctransfer
    /// module to.
    Transfer {
        /// exisiting channel to send the tokens over
        channel_id: String,
        /// address on the remote chain to receive these tokens
        to_address: String,
        /// packet data only supports one coin
        /// https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/ibc/applications/transfer/v1/transfer.proto#L11-L20
        amount: Coin,
        /// when packet times out, measured on remote chain
        timeout: IbcTimeout,
    }
}

/// In IBC each package must set at least one type of timeout:
/// the timestamp or the block height. Using this rather complex enum instead of
/// two timeout fields we ensure that at least one timeout is set.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct IbcTimeout {
    block: Option<IbcTimeoutBlock>,
    timestamp: Option<Timestamp>,
}
```

Note the `to_address` is likely not a valid `Addr`, as it uses the bech32 prefix
of the _receiving_ chain. In addition to the info you need in `BankMsg::Send`,
you need to define the `channel` to send upon as well as a timeout specified
either in block height or block time (or both). If the packet is not relayed
before the timeout passes (measured on the receiving chain), you can request
your tokens back.

## Writing New Protocols

However, we go beyond simply _using_ existing IBC protocols, and allow you to
_implement_ your own ICS protocols inside the contract. A good example to
understand this is the
[`cw20-ics20` contract](https://github.com/CosmWasm/cosmwasm-plus/tree/v0.6.0-beta1/contracts/cw20-ics20)
included in the `cosmwasm-plus` repo. This contract speaks the `ics20-1`
protocol to an external blockchain just as if it were the `ibctransfer` module
in Go. However, we can implement any logic we want there and even hot-load it on
a running blockchain.

This particular contract above accepts
[cw20 tokens](https://github.com/CosmWasm/cosmwasm-plus/tree/v0.6.0-beta1/packages/cw20)
and sends those to a remote chain, as well as receiving the tokens back and
releasing the original cw20 token to a new owner. It does not (yet) allow
minting coins originating from the remote chain. I recommend opening up the
source code for that contract and refering to it when you want a concrete
example for anything discussed below.

In order to enable IBC communication, a contract must expose the following 6
entry points. Upon detecting such an "IBC-Enabled" contract, the
[`x/wasm` runtime](https://github.com/CosmWasm/wasmd) will automatically bind a
port for this contract (`wasm.<contract-address>`), which allows a relayer to
create channels between this contract and another chain. Once channels are
created, the contract will process all packets and receipts.

### Channel Lifecycle

You should first familiarize yourself with the
[4 step channel handshake protocol](https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#channel-lifecycle-management)
from the IBC spec. After realizing that it was 2 slight variants of 2 steps, we
simplified the interface for the contracts. Each side will receive 2 calls to
establish a new channel, and returning an error in any of the steps will abort
the handshake. Below we will refer to the chains as A and B - A is where the
handshake initialized at.

#### Channel Open

The first step of a handshake on either chain is `ibc_channel_open`, which
combines `ChanOpenInit` and `ChanOpenTry` from the spec. The only valid action
of the contract is to accept the channel or reject it. This is generally based
on the ordering and version in the `IbcChannel` information, but you could
enforce other constraints as well:

```rust
#[entry_point]
/// enforces ordering and versioning constraints
pub fn ibc_channel_open(deps: DepsMut, env: Env, msg: IbcChannelOpenMsg) -> StdResult<()> { }
```

This is the
[IbcChannel structure](https://github.com/CosmWasm/cosmwasm/blob/v0.15.0/packages/std/src/ibc.rs#L117-L128)
used heavily in the handshake process:

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IbcChannel {
    pub endpoint: IbcEndpoint,
    pub counterparty_endpoint: IbcEndpoint,
    pub order: IbcOrder,
    pub version: String,
    /// CounterpartyVersion can be None when not known this context, yet
    pub counterparty_version: Option<String>,
    /// The connection upon which this channel was created. If this is a multi-hop
    /// channel, we only expose the first hop.
    pub connection_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IbcEndpoint {
    pub port_id: String,
    pub channel_id: String,
}
```

This `IbcChannel` value has to be wrapped in the `IbcChannelOpenMsg` type.

```rust
let msg = IbcChannelOpenMsg::new_init(channel);
// or
let msg = IbcChannelOpenMsg::new_try(channel, counterparty_version);
```

Note that neither `counterparty_version` nor `counterparty_endpoint` is set in
`ibc_channel_open` for chain A. Chain B should enforce any
`counterparty_version` constraints in `ibc_channel_open`. Chain A must enforce
`counterparty_version` or `counterparty_endpoint` restrictions in
`ibc_channel_connect`.

(Just test if the `counterparty_version` field is `Some(x)` in both calls and
then enforce the counterparty restrictions if set. That will check these once at
the proper place for both chain A and chain B).

You should save any state only in `ibc_channel_connect` once the channel has
been approved by the remote side.

#### Channel Connect

Once both sides have returned `Ok()` to `ibc_channel_open`, we move onto the
second step of the handshake, which is equivalent to `ChanOpenAck` and
`ChanOpenConfirm` from the spec:

```rust
#[entry_point]
/// once it's established, we may take some setup action
pub fn ibc_channel_connect(
    deps: DepsMut,
    env: Env,
    msg: IbcChannelConnectMsg,
) -> StdResult<IbcBasicResponse> { }
```

At this point, it is expected that the contract updates its internal state and
may return `CosmosMsg` in the `Reponse` to interact with other contracts, just
like in `execute`. In particular, you will most likely want to store the local
channel_id (`channel.endpoint.channel_id`) in the contract's storage, so it
knows what open channels it has (and can expose those via queries or maintain
state for each one).

Once this has been called, you may expect to send and receive any number of
packets with the contract. The packets will only stop once the channel is closed
(which may never happen).

### Channel Close

A contract may request to close a channel that belongs to it via the following
`CosmosMsg::Ibc`:

```rust
pub enum IbcMsg {
    /// This will close an existing channel that is owned by this contract.
    /// Port is auto-assigned to the contract's IBC port
    CloseChannel { channel_id: String },
}
```

Once a channel is closed, whether due to an IBC error, at our request, or at the
request of the other side, the following callback is made on the contract, which
allows it to take appropriate cleanup action:

```rust
#[entry_point]
pub fn ibc_channel_close(
    deps: DepsMut,
    env: Env,
    msg: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> { }
```

### Packet Lifecycle

Unfortunately the
[IBC spec on Packet Lifecycle](https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#packet-flow--handling)
is missing all useful diagrams, but it may provide some theoretical background
for this text if you wish to look.

In short, IBC allows us to send packets from chain A to chain B and get a
response from them. The first step is the contract/module in chain A requesting
to send a packet. This is then relayed to chain B, where it "receives" the
packet and calculates an "acknowledgement" (which may contain a success result
or an error message, as opaque bytes to be interpreted by the sending contract).
The acknowledgement is then relayed back to chain A, completing the cycle.

In some cases, the packet may never be delivered, and if it is proven not to be
delivered before the timeout, this can abort the packet, calling the "timeout"
handler on chain A. In this case, chain A sends and later gets "timeout". No
"receive" nor "acknowledgement" callbacks are ever executed.

#### Sending a Packet

In order to send a packet, a contract can simply return `IbcMsg::SendPacket`
along with the channel over which to send the packet (which you saved in
`ibc_channel_connect`), as well as opaque data bytes to be interpreted by the
other side. You must also return a timeout either as block height or block time
of the remote chain, just like in the ICS20 `Transfer` messages above:

```rust
pub enum IbcMsg {
    /// Sends an IBC packet with given data over the existing channel.
    /// Data should be encoded in a format defined by the channel version,
    /// and the module on the other side should know how to parse this.
    SendPacket {
        channel_id: String,
        data: Binary,
        /// when packet times out, measured on remote chain
        timeout: IbcTimeout,
    },
}
```

For the content of the `data` field, we recommend that you model it on the
format of `ExecuteMsg` (an enum with serde) and encode it via
`cosmwasm_std::to_binary(&packet_msg)?`. This is the approach for a new protocol
you develop with cosmwasm contracts. If you are working with an existing
protocol, please read their spec and create the proper type along with JSON or
Protobuf encoders for it as the protocol requires.

#### Receiving a Packet

After a contract on chain A sends a packet, it is generally processed by the
contract on chain B on the other side of the channel. This is done by executing
the following entry point on chain B:

```rust
#[entry_point]
pub fn ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, Never> {
    // ...
}
```

This is a very special entry point as it has a unique workflow. (Please see the
[Acknowledging Errors section](#Acknowledging-Errors) below to understand it
fully).

Also note the different return response here (`IbcReceiveResponse` rather than
`IbcBasicResponse`). This is because it has an extra field
`acknowledgement: Binary`, which must be filled out. All successful message must
return an encoded `Acknowledgement` response in this field, that can be parsed
by the sending chain.

The
[`IbcPacket` structure](https://github.com/CosmWasm/cosmwasm/blob/v0.15.0/packages/std/src/ibc.rs#L176-L187)
contains all information needed to process the receipt. This info has already
been verified by the core IBC modules via light client and merkle proofs. It
guarantees all metadata in the `IbcPacket` structure is valid, and the `data`
field was written on the remote chain. Furthermore, it guarantees that the
packet is processed at most once (zero times if it times out). Fields like
`dest.channel_id` and `sequence` have a similar trust level to `MessageInfo`,
which we use to authorize normal transactions. The `data` field should be
treated like the `ExecuteMsg` data, which is only as valid as the entity that
signed it.

You can generally ignore `timeout_*` (this entry point is only called if it
hasn't yet timed out) and `sequence` (which is used by the IBC framework to
avoid duplicates). I generally use `dest.channel_id` like `info.sender` to
authenticate the packet, and parse `data` into a `PacketMsg` structure, using
the same encoding rules as we discussed in the last section. After that you can
process `PacketMsg` more or less like an `ExecuteMsg`, including calling into
other contracts.

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IbcPacket {
    /// The raw data send from the other side in the packet
    pub data: Binary,
    /// identifies the channel and port on the sending chain.
    pub src: IbcEndpoint,
    /// identifies the channel and port on the receiving chain.
    pub dest: IbcEndpoint,
    /// The sequence number of the packet on the given channel
    pub sequence: u64,
    pub timeout: IbcTimeout,
}
```

`IbcPacketReceiveMsg` is currently a wrapper around `IbcPacket`.

##### Acknowledging Errors

A major issue that is unique to `ibc_packet_receive` is that it is expected to
often reject an incoming packet, yet it cannot abort the transaction. We
actually expect all state changes from the contract (as well as dispatched
messages) to be reverted when the packet is rejected, but the transaction to
properly commit an acknowledgement with encoded error. In other words, this "IBC
Handler" will error and revert, but the "IBC Router" must succeed and commit an
acknowledgement message (that can be parsed by the sending chain as an error).

The atomicity issue was first
[analyzed in the Cosmos SDK implementation](https://github.com/cosmos/ibc-go/issues/68)
and refined into
[changing semantics of the OnRecvPacket SDK method](https://github.com/cosmos/ibc-go/issues/91),
which was
[implemented in April 2021](https://github.com/cosmos/ibc-go/pull/107), likely
to be released with Cosmos SDK 0.43 or 0.44. Since we want the best,
future-proof interface for contracts, we will use an approach inspired by that
work, and add an adapter in `wasmd` until we can upgrade to a Cosmos SDK version
that implements this.

After quite some
[discussion on how to encode the errors](https://github.com/CosmWasm/cosmwasm/issues/762),
we struggled to map this idea to the CosmWasm model. However, we also discovered
a deep similarity between these requirements and the
[submessage semantics](./SEMANTICS.md#submessages). It just requires some
careful coding on the contract developer's side to not throw errors. This
produced 3 suggestions on how to handle errors and rollbacks _inside
`ibc_packet_receive`_

1. If the message doesn't modify any state directly, you can simply put the
   logic in a closure, and capture errors, converting them into error
   acknowledgements. This would look something like the
   [main dispatch loop in `ibc-reflect`](https://github.com/CosmWasm/cosmwasm/blob/cd784cd1148ee395574f3e564f102d0d7b5adcc3/contracts/ibc-reflect/src/contract.rs#L217-L248):

   ```rust
   pub fn ibc_packet_receive(
       deps: DepsMut,
       _env: Env,
       msg: IbcPacketReceiveMsg,
   ) -> Result<IbcReceiveResponse, Never> {
       (|| {
           // which local channel did this packet come on
           let caller = packet.dest.channel_id;
           let msg: PacketMsg = from_slice(&packet.data)?;
           match msg {
               PacketMsg::Dispatch { msgs } => receive_dispatch(deps, caller, msgs),
               PacketMsg::WhoAmI {} => receive_who_am_i(deps, caller),
               PacketMsg::Balances {} => receive_balances(deps, caller),
           }
       })()
       .or_else(|e| {
           // we try to capture all app-level errors and convert them into
           // acknowledgement packets that contain an error code.
           let acknowledgement = encode_ibc_error(format!("invalid packet: {}", e));
           Ok(IbcReceiveResponse {
               acknowledgement,
               submessages: vec![],
               messages: vec![],
               attributes: vec![],
           })
       })
   }
   ```

2. If we modify state with an external call, we need to wrap it in a
   `submessage` and capture the error. This approach requires we use _exactly
   one_ submessage. If we have multiple, we may commit #1 and rollback #2 (see
   example 3 for that case). The main point is moving `messages` to
   `submessages` and reformating the error in `reply`. Note that if you set the
   `Response.data` field in `reply` it will override the acknowledgement
   returned from the parent call. (See
   [bottom of reply section](./SEMANTICS.md#handling-the-reply)). You can see a
   similar example in how
   [`ibc-reflect` handles `receive_dispatch`](https://github.com/CosmWasm/cosmwasm/blob/eebb9395ccf315320e3f2fcc526ee76788f89174/contracts/ibc-reflect/src/contract.rs#L307-L336).
   Note how we use a unique reply ID for this and use that to catch any
   execution failure and return an error acknowledgement instead:

   ```rust
   fn receive_dispatch(
       deps: DepsMut,
       caller: String,
       msgs: Vec<CosmosMsg>,
   ) -> StdResult<IbcReceiveResponse> {
       // what is the reflect contract here
       let reflect_addr = accounts(deps.storage).load(caller.as_bytes())?;

       // let them know we're fine
       let acknowledgement = to_binary(&AcknowledgementMsg::<DispatchResponse>::Ok(()))?;
       // create the message to re-dispatch to the reflect contract
       let reflect_msg = ReflectExecuteMsg::ReflectMsg { msgs };
       let wasm_msg = wasm_execute(reflect_addr, &reflect_msg, vec![])?;

       // we wrap it in a submessage to properly report errors
       let sub_msg = SubMsg {
           id: RECEIVE_DISPATCH_ID,
           msg: wasm_msg.into(),
           gas_limit: None,
           reply_on: ReplyOn::Error,
       };

       Ok(IbcReceiveResponse {
           acknowledgement,
           submessages: vec![sub_msg],
           messages: vec![],
           attributes: vec![attr("action", "receive_dispatch")],
       })
   }

   #[entry_point]
   pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> StdResult<Response> {
      match (reply.id, reply.result) {
         (RECEIVE_DISPATCH_ID, ContractResult::Err(err)) => Ok(Response {
            data: Some(encode_ibc_error(err)),
            ..Response::default()
         }),
         (INIT_CALLBACK_ID, ContractResult::Ok(response)) => handle_init_callback(deps, response),
         _ => Err(StdError::generic_err("invalid reply id or result")),
      }
   }
   ```

3. For a more complex case, where we are modifying local state and possibly
   sending multiple messages, we need to do a self-call via submessages. What I
   mean is that we create a new `ExecuteMsg` variant, which returns an error if
   called by anyone but the contract itself
   (`if info.sender != env.contract.address { return Err() }`). When receiving
   the IBC packet, we can create a submessage with `ExecuteMsg::DoReceivePacket`
   and any args we need to pass down.

   `DoReceivePacket` should return a proper acknowledgement payload on success.
   And return an error on failure, just like a normal `execute` call. However,
   here we capture both success and error cases in the `reply` handler (use
   `ReplyOn::Always`). For success, we return this data verbatim to be set as
   the packet acknowledgement, and for errors, we encode them as we did above.
   There is not any example code using this (yet), but it is just recombining
   pieces we already have. For clarity, the `reply` statement should look
   something like:

   ```rust
   #[entry_point]
   pub fn reply(_deps: DepsMut, _env: Env, reply: Reply) -> StdResult<Response> {
       if reply.id != DO_IBC_RECEIVE_ID {
           return Err(StdError::generic_err("invalid reply id"));
       }
       let data = match reply.result {
           ContractResult::Ok(response) => response.data,
           ContractResult::Err(err) => Some(encode_ibc_error(err)),
       };
       Ok(Response {
           data,
           ..Response::default()
       })
   }
   ```

##### Standard Acknowledgement Envelope

Although the ICS spec leave the actual acknowledgement as opaque bytes, it does
provide a recommendation for the format you can use, allowing contracts to
easily differentiate between success and error (and allow IBC explorers to label
such packets without knowing every protocol).

It is defined as part of the
[ICS4 - Channel Spec](https://github.com/cosmos/cosmos-sdk/blob/v0.42.4/proto/ibc/core/channel/v1/channel.proto#L134-L147).

```proto
message Acknowledgement {
  // response contains either a result or an error and must be non-empty
  oneof response {
    bytes  result = 21;
    string error  = 22;
  }
}
```

Although it suggests this is a Protobuf object, the ICS spec doesn't define
whether to encode it as JSON or Protobuf. In the ICS20 implementation, this is
JSON encoded when returned from a contract. In ICS27, the authors are discussing
using a Protobuf-encoded form of this structure.

Note that it leaves the actual success response as app-specific bytes where you
can place anything, but does provide a standard way for an observer to check
success-or-error. If you are designing a new protocol, I encourage you to use
this struct in either of the encodings as the acknowledgement envelope.

You can find a
[CosmWasm-compatible definition of this format](https://github.com/CosmWasm/cosmwasm-plus/blob/v0.6.0-beta1/contracts/cw20-ics20/src/ibc.rs#L52-L72)
as part of the `cw20-ics20` contract, along with JSON-encoding. Protobuf
encoding version can be produced upon request.

#### Receiving an Acknowledgement

If chain B successfully received the packet (even if the contract returned an
error message), chain A will eventually get an acknowledgement:

```rust
#[entry_point]
/// never should be called as we do not send packets
pub fn ibc_packet_ack(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketAckMsg,
) -> StdResult<IbcBasicResponse> { }
```

The
[`IbcAcknowledgement` structure](https://github.com/CosmWasm/cosmwasm/blob/v0.15.0/packages/std/src/ibc.rs#L195-L200)
contains both the original packet that was sent as well as the acknowledgement
bytes returned from executing the remote contract. You can use the
`original_packet` to
[map it the proper handler](https://github.com/CosmWasm/cosmwasm/blob/378b029707ebaed4505b3666c620bed69ff9a682/contracts/ibc-reflect-send/src/ibc.rs#L111-L136)
(after parsing your custom data format), and parse the `acknowledgement` there,
to determine how to respond:

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IbcAcknowledgement {
    pub acknowledgement: Binary,
    pub original_packet: IbcPacket,
}
```

On success, you will want to commit the pending state. For some contracts like
`cw20-ics20`, you accept the tokens before sending the packet, so no need to
commit any more state. On other contracts, you may want to store the data
returned as part of the acknowledgement (like
[storing the remote address after calling "WhoAmI"](https://github.com/CosmWasm/cosmwasm/blob/v0.14.0-beta4/contracts/ibc-reflect-send/src/ibc.rs#L157-L192)
in our simple `ibc-reflect` example.

On error, you will want to revert any state that was pending based on the
packet. For example, in ics20, if the
[remote chain rejects the packet](https://github.com/CosmWasm/cw-plus/blob/v0.10.0/contracts/cw20-ics20/src/ibc.rs#L248),
we must
[return the funds to the original sender](https://github.com/CosmWasm/cw-plus/blob/v0.10.0/contracts/cw20-ics20/src/ibc.rs#L291-L313).

#### Handling Timeouts

If the packet was not received on chain B before the timeout, we can be certain
that it will never be processed there. In such a case, a relayer can return a
timeout proof to cancel the pending packet. In such a case the calling contract
will never get `ibc_packet_ack`, but rather `ibc_packet_timeout`. One of the two
calls will eventually get called for each packet that is sent as long as there
is a functioning relayer. (In the absence of a functioning relayer, it will
never get a response).

The timeout callback looks like this:

```rust
#[entry_point]
/// never should be called as we do not send packets
pub fn ibc_packet_timeout(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {}
```

It is generally handled just like the error case in `ibc_packet_ack`, reverting
the state change from sending the packet (eg. if we send tokens over ICS20, both
[an ack failure](https://github.com/CosmWasm/cw-plus/blob/v0.10.0/contracts/cw20-ics20/src/ibc.rs#L248)
as well as
[a timeout](https://github.com/CosmWasm/cw-plus/blob/v0.10.0/contracts/cw20-ics20/src/ibc.rs#L261)
will return those tokens to the original sender. In fact they both dispatch to
the same `on_packet_failure` function).

Note that like `ibc_packet_ack`, we get the original packet we sent, which must
contain all information needed to revert itself. Thus the ICS20 packet contains
the original sender address, even though that is unimportant in the receiving
chain.
