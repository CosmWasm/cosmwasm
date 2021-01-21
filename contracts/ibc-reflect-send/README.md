# Ibc Reflect Send Contract

This is a simple contract to demonstrate using contracts using IBC messages. The
first case we build is to simulate the `reflect` contract on another chain. That
is, you can send a message over IBC to the reflect contract and it will
"reflect" that message on the remote chain as if it sent it.

This is inspired by
[ICS27](https://github.com/chainapsis/cosmos-sdk-interchain-account/tree/master/x/ibc-account/spec)
and uses a similar workflow, but we use different messages to make it easier for
building with cosmwasm. In the future we could try to implement the ICS27 spec
byte-for-byte compatible inside a CosmWasm contract, but that is not the
intention here.

## Workflow

This is the contract from the sending chain which corresponds to
the [`ibc-reflect`](../ibc-reflect) "factory" contract on the receiving chain.

The `ibc-reflect-send` contract has one admin and binds a port
on `init`. You can bind any number of channels to this contract,
each one linked to a `ibc-reflect` contract on a remote chain.
It does not accept any incoming packets over the channel, but rather
sends packets (the opposite of `ibc-reflect`).

Upon a successful connection, it will send a `WhoAmI` packet
to find the address on the remote chain and store it locally to
answer all queries.

It contains 3 methods in `HandleMsg`:

* `UpdateAdmin` - to change which account can send
* `SendMsgs` - to send a packet full of `CosmosMsg` to the remote chain
  over the given channel.
* `UpdateRemoteBalance` - this will send `Balances` packets
  and store the info locally
  
It contains 2 methods in `QueryMsg`:

* `Admin` - to show current admin
* `ListChannels` - to list all open channels (you must select
  one for each packet you send), along with the account address
  on the remote chain, and balances (if you have some)
  
## Protocol

See [`ibc-reflect`](../ibc-reflect) for a full description of the
IBC packet protocol
