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
`counterparty_version` constraints in `ibc_channel_open`. (So test if the field
is `None` before enforcing any logic).

You should save any state related to `counterparty_endpoint` only in
`ibc_channel_connect` when it is fixed.

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
like in `execute`.

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

Once a channel is closed, due to error, our request, or request of the other
side, the following callback is made on the contract, which allows it to take
appropriate cleanup action:

```rust
#[entry_point]
pub fn ibc_channel_close(
    deps: DepsMut,
    env: Env,
    channel: IbcChannel,
) -> StdResult<IbcBasicResponse> { }
```

### Packet Lifecycle
