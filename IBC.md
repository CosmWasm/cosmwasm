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

### Packet Lifecycle
