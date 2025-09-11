# Ibc Reflect Contract

This is a simple contract to demonstrate using contracts using IBC messages. The
first case we build is to simulate the `reflect` contract on another chain. That
is, you can send a message over IBC to the `reflect` contract, and it will
"reflect" that message on the remote chain as if it sent it.

This is inspired by
[ICS27](https://github.com/chainapsis/cosmos-sdk-interchain-account/tree/master/x/ibc-account/spec)
and uses a similar workflow, but we use different messages to make it easier for
building with cosmwasm. In the future we could try to implement the ICS27 spec
byte-for-byte compatible inside a CosmWasm contract, but that is not the
intention here.

## Workflow

This requires 2 contracts on the remote chain. The first is this contract, which
is essentially a factory. The second is the default [`reflect`](../reflect)
contract, which allows the factory to control multiple independent accounts.

The factory will handshake and accept connections from any attempt that uses the
`ibc-reflect` "version" for the protocol negotiation. This will create a new
channel. Once the connection is established (in the `ibc_channel_connect` entry
point), it will create a new `reflect` contract instance. The reflect `code_id`
must be set when initializing the factory. This `reflect` contract address will
be saved and connected to the channel.

Once the channel is fully established and the `reflect` contract instantiated it
will expect a `RunTx` message, which contains `Vec<CosmosMsg>`. When this
message is received, it will execute it on the `reflect` contract, performing
the requested action on behalf of the remote user.

## Issues

- How to set the return value from the execution properly? We return them async
- How to handle errors properly?
- How to send packets in the proper format. I guess we need an ibc-reflect-send
  contract on the origin chain?

## Protocol

We require version `ibc-reflect-v1` when making the ibc handshake.

The packets sent look like:

```rust
pub enum PacketMsg {
  Dispatch { msgs: Vec<CosmosMsg> },
  WhoAmI {},
  Balances {},
}
```

That is, one of the following:

```json
[
  { "dispatch": ["large struct here.."] },
  { "who_am_i": {} },
  { "balances": {} }
]
```

The success responses look like one of the following:

Dispatch:

```json
{
  "ok": null
}
```

WhoAmI:

```json
{
  "account": "wasm12skc92jiowf8hwfhofqfh225ss"
}
```

Balances:

```json
{
  "account": "wasm12skc92jiowf8hwfhofqfh225ss",
  "balances": [
    { "amount": "12345678", "denom": "uatom" },
    { "amount": "777777", "denom": "tgrd" }
  ]
}
```

The error ack packet always looks like this:

```json
{
  "error": "invalid packet: <detailed error message>"
}
```
