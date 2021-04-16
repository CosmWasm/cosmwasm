# IBC interfaces for CosmWasm contracts

If you import `cosmwasm-std` with the `stargate` feature flag, it will expose a
number of IBC-related functionality. This requires that the host chain is
running an IBC-enabled version of `wasmd`, that is `v0.16.0` or higher. You will
get an error when you upload the contract if the chain doesn't support this
functionality.

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
        /// block after which the packet times out.
        /// at least one of timeout_block, timeout_timestamp is required
        timeout_block: Option<IbcTimeoutBlock>,
        /// block timestamp (nanoseconds since UNIX epoch) after which the packet times out.
        /// See https://golang.org/pkg/time/#Time.UnixNano
        /// at least one of timeout_block, timeout_timestamp is required
        timeout_timestamp: Option<u64>,
    }
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
[wasmd runtime](https://github.com/CosmWasm/wasmd) will automatically bind a
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
pub fn ibc_channel_open(deps: DepsMut, env: Env, channel: IbcChannel) -> StdResult<()> { }
```

This is the
[IbcChannel structure](https://github.com/CosmWasm/cosmwasm/blob/v0.14.0-beta4/packages/std/src/ibc.rs#L70-L81)
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
    channel: IbcChannel,
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
`CosmosMsg`:

```rust
pub enum IbcMsg {
    /// This will close an existing channel that is owned by this contract.
    /// Port is auto-assigned to the contracts' ibc port
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
    channel: IbcChannel,
) -> StdResult<IbcBasicResponse> { }
```

### Packet Lifecycle

Unfortunately the
[IBC spec on Pakcet Lifecycle](https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics#packet-flow--handling)
is missing all useful diagrams, but it may provide some theoretical background
for this text if you wish to look.

In short, IBC allows us to send packets from chain A to chain B and get a
response from them. The first step is the contract/module in chain A requesting
to send a packet. This is then relayed to chain B, where it "receives" the
packet and calculates an "acknowledgement" (which may contain a success result
or an error message, as opaque bytes to be interpretted by the sending
contract). The acknowledgement is then relayed back to chain A, completing the
cycle.

In some cases, the packet may never be delivered, and if it is proven not to be
delivered before the timeout period, this can abort the packet, calling the
"timeout" handler on chain A. In this case, chain A sends and later gets
"timeout". No "receive" nor "acknowledgement" callbacks are ever executed.

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
        /// block height after which the packet times out.
        /// at least one of timeout_block, timeout_timestamp is required
        timeout_block: Option<IbcTimeoutBlock>,
        /// block timestamp (nanoseconds since UNIX epoch) after which the packet times out.
        /// See https://golang.org/pkg/time/#Time.UnixNano
        /// at least one of timeout_block, timeout_timestamp is required
        timeout_timestamp: Option<u64>,
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
the following callback on chain B:

```rust
#[entry_point]
pub fn ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    packet: IbcPacket,
) -> StdResult<IbcReceiveResponse> { }
```

Note the different return response here (`IbcReceiveResponse` rather than
`IbcBasicResponse`)? This is because it has an extra field
`acknowledgement: Binary`, which must be filled out. That is the response bytes
that will be returned to the original contract, informing it of failure or
success. (Note: this is vague as it will be refined in the next PR)

Here is the
[`IbcPacket` structure](https://github.com/CosmWasm/cosmwasm/blob/v0.14.0-beta4/packages/std/src/ibc.rs#L129-L146)
that contains all information needed to process the receipt. You can generally
ignore timeout (this is only called if it hasn't yet timed out) and sequence
(which is used by the IBC framework to avoid duplicates). I generally use
`dest.channel_id` like `info.sender` to authenticate the packet, and parse
`data` into a `PacketMsg` structure, using the same encoding rules as we
discussed in the last section.

After that you can process `PacketMsg` more or less like an `ExecuteMsg`,
including calling into other contracts. The only major difference is that you
must return Acknowledgement bytes in the protocol-specified format

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
    /// block height after which the packet times out.
    /// at least one of timeout_block, timeout_timestamp is required
    pub timeout_block: Option<IbcTimeoutBlock>,
    /// block timestamp (nanoseconds since UNIX epoch) after which the packet times out.
    /// See https://golang.org/pkg/time/#Time.UnixNano
    /// at least one of timeout_block, timeout_timestamp is required
    pub timeout_timestamp: Option<u64>,
}
```

TODO: explain how to handle/parse errors (As part of
https://github.com/CosmWasm/cosmwasm/issues/762)

##### Standard Acknowledgement Format

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
JSON encoded when returned from a contract. Given that, we will consider this
structure, JSON-encoded, to be the "standard" acknowledgement format.

You can find a
[CosmWasm-compatible definition of this format](https://github.com/CosmWasm/cosmwasm-plus/blob/v0.6.0-beta1/contracts/cw20-ics20/src/ibc.rs#L52-L72)
as part of the `cw20-ics20` contract.

#### Receiving an Acknowledgement

If chain B successfully received the packet (even if the contract returned an
error message), chain A will eventually get an acknowledgement:

```rust
#[entry_point]
/// never should be called as we do not send packets
pub fn ibc_packet_ack(
    deps: DepsMut,
    env: Env,
    ack: IbcAcknowledgement,
) -> StdResult<IbcBasicResponse> { }
```

The
[`IbcAcknowledgement` structure](https://github.com/CosmWasm/cosmwasm/blob/v0.14.0-beta4/packages/std/src/ibc.rs#L148-L152)
contains both the original packet that was sent as well as the acknowledgement
bytes returned from executing the remote contract. You can use the
`original_packet` to
[map it the proper handler](https://github.com/CosmWasm/cosmwasm/blob/v0.14.0-beta4/contracts/ibc-reflect-send/src/ibc.rs#L114-L138)
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
[remote chain rejects the packet](https://github.com/CosmWasm/cosmwasm-plus/blob/v0.6.0-beta1/contracts/cw20-ics20/src/ibc.rs#L246),
we must
[return the funds to the original sender](https://github.com/CosmWasm/cosmwasm-plus/blob/v0.6.0-beta1/contracts/cw20-ics20/src/ibc.rs#L291-L317).

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
    packet: IbcPacket,
) -> StdResult<IbcBasicResponse> {}
```

It is generally handled just like the error case in `ibc_packet_ack`, reverting
the state change from sending the packet (eg. if we send tokens over ICS20, both
[an ack failure](https://github.com/CosmWasm/cosmwasm-plus/blob/v0.6.0-beta1/contracts/cw20-ics20/src/ibc.rs#L246)
as well as
[a timeout](https://github.com/CosmWasm/cosmwasm-plus/blob/v0.6.0-beta1/contracts/cw20-ics20/src/ibc.rs#L258)
will return those tokens to the original sender. In fact they both dispatch to
the same `on_packet_failure` function).

Note that like `ibc_packet_ack`, we get the original packet we sent, which must
contain all information needed to revert itself. Thus the ICS20 packet contains
the original sender address, even though that is unimportant in the receiving
chain.
