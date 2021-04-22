# Contract Semantics

This document aims to clarify the semantics of how a CosmWasm contract interacts
with its environment. There are two main types of actions: _mutating_ actions,
which receive `DepsMut` and are able to modify the state of the blockchain, and
_query_ actions, which are run on a single node with read-only access to the
data.

## Execution

In the section below, we will discuss how the `execute` call works, but the same
semantics apply to any other _mutating_ action - `instantiate`, `migrate`,
`sudo`, etc.

### SDK Context

Before looking at CosmWasm, we should look at the (somewhat under-documented)
semantics enforced by the blockchain framework we integrate with - the
[Cosmos SDK](https://v1.cosmos.network/sdk). It is based upon the
[Tendermint BFT](https://tendermint.com/core/) Consensus Engine. Let us first
look how they process transactions before they arrive in CosmWasm (and after
they leave).

First, the Tendermint engine will seek 2/3+ consensus on a list of transactions
to be included in the next block. This is done _without executing them_. They
are simply subjected to a minimal pre-filter by the Cosmos SDK module, to ensure
they are validly formatted transactions, with sufficient gas fees, and signed by
an account with sufficient fees to pay it. Notably, this means many transactions
that error may be included in a block.

Once a block is committed (typically every 5s or so), the transactions are then
fed to the Cosmos SDK sequentially in order to execute them. Each one returns a
result or error along with event logs, which are recorded in the `TxResults`
section of the next block. The `AppHash` (or merkle proof or blockchain state)
after executing the block is also included in the next block.

The Cosmos SDK `BaseApp` handles each transaction in an isolated context. It
first verifies all signatures and deducts the gas fees. It sets the "Gas Meter"
to limit the execution to the amount of gas paid for by the fees. Then it makes
an isolated context to run the transaction. This allows the code to read the
current state of the chain (after the last transaction finished), but it only
writes to a cache, which may be committed or rolled back on error.

A transaction may consist of multiple messages and each one is executed in turn
under the same context and same gas limit. If all messages succeed, the context
will be committed to the underlying blockchain state and the results of all
messages will be stored in the `TxResult`. If one message fails, all later
messages are skipped and all state changes are reverted. This is very important
for atomicity. That means Alice and Bob can both sign a transaction with 2
messages: Alice pays Bob 1000 ATOM, Bob pays Alice 50 ETH, and if Bob doesn't
have the funds in his account, Alice's payment will also be reverted. This is
just like a DB Transaction typically works.

[`x/wasm`](https://github.com/CosmWasm/wasmd/tree/master/x/wasm) is a custom
Cosmos SDK module, which processes certain messages and uses them to upload,
instantiate, and execute smart contracts. In particular, it accepts a properly
signed
[`MsgExecuteContract`](https://github.com/CosmWasm/wasmd/blob/master/proto/cosmwasm/wasm/v1beta1/tx.proto#L76-L89),
routes it to
[`Keeper.Execute`](https://github.com/CosmWasm/wasmd/blob/master/x/wasm/keeper/keeper.go#L311-L355),
which loads the proper smart contract and calls `execute` on it. Note that this
method may either return a success (with data and events) or an error. In the
case of an error here, it will revert the entire transaction in the block. This
is the context we find ourselves in when our contract receives the `execute`
call.

### Basic Execution

When we implement a contract, we provide the following entry point:


```rust
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> { }
```

With `DepsMut`, this can read and write to the backing `Storage`, as well as use the `Api` to validate addresses,
and `Query` the state of other contracts or native modules. Once it is done, it returns either `Ok(Response)`
or `Err(ContractError)`. Let's examine what happens next:

If it returns `Err`, this error is converted to a string representation (`err.to_string()`), and this is returned
to the SDK module. *All state changes are reverted* and `x/wasm` returns this error message, which will *generally*
(see submessage exception below) abort the transaction, and return this same error message to the external caller.

If it returns `Ok`, the `Response` object is parsed and processed. Let's look at the parts here:

```rust
pub struct Response<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    /// Optional list of "subcalls" to make. These will be executed in order
    /// (and this contract's subcall_response entry point invoked)
    /// *before* any of the "fire and forget" messages get executed.
    pub submessages: Vec<SubMsg<T>>,
    /// After any submessages are processed, these are all dispatched in the host blockchain.
    /// If they all succeed, then the transaction is committed. If any fail, then the transaction
    /// and any local contract state changes are reverted.
    pub messages: Vec<CosmosMsg<T>>,
    /// The attributes that will be emitted as part of a "wasm" event
    pub attributes: Vec<Attribute>,
    pub data: Option<Binary>,
}
```

In the Cosmos SDK, a transaction returns a number of events to the user, along with an optional data "result". This
result is hashed into the next block hash to be provable and can return some essential state (although in general
client apps rely on Events more). This result is more commonly used to pass results between contracts or modules in
the sdk.

If the contract sets `data`, this will be returned in the `result` field. `attributes` is a list of `{key, value}`
pairs which will be [appended to a default event](https://github.com/CosmWasm/wasmd/blob/master/x/wasm/types/types.go#L302-L321).
The final result looks like this to the client:

```json
{
  "type": "wasm",
  "attributes": [
    {"key":  "contract_addr", "value":  "cosmos1234567890qwerty"},
    {"key":  "custom-key-1", "value":  "custom-value-1"},
    {"key":  "custom-key-2", "value":  "custom-value-2"}
  ]
}
```

### Dispatching Messages

Now let's move onto the `messages` field. Some contracts are fine only talking with themselves, such as a cw20
contract just adjusting it's balances on transfers. But many want to move tokens (native or cw20) or call into
other contracts for more complex actions. This is where messages come in. We return 
[`CosmosMsg`, which is a serializable representation](https://github.com/CosmWasm/cosmwasm/blob/v0.14.0-beta4/packages/std/src/results/cosmos_msg.rs#L18-L40)
of any external call the contract can make. It looks something like this (with `stargate` feature flag enabled):

```rust
pub enum CosmosMsg<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    Bank(BankMsg),
    /// This can be defined by each blockchain as a custom extension
    Custom(T),
    Staking(StakingMsg),
    Distribution(DistributionMsg),
    Stargate {
        type_url: String,
        value: Binary,
    },
    Ibc(IbcMsg),
    Wasm(WasmMsg),
}
```

If a contract returns two messages - M1 and M2, these will both be parsed and executed in `x/wasm` 
*with the permissions of the contract* (meaning `info.sender` will be the contract not the original caller).
If they return success, they will emit a new event with the custom attributes, the `data` field will be ignored,
and any messages they return will also be processed. If they return an error, the parent call will return an error,
thus rolling back state of the whole transaction.

Note that the messages are executed *depth-first*. This means if contract A returns M1 (`WasmMsg::Execute`) and 
M2 (`BankMsg::Send`), and contract B (from the `WasmMsg::Execute`) returns N1 and N2 (eg. `StakingMsg` and `DistributionMsg`),
the order of execution would be **M1, N1, N2, M2**.

### Submessages

Reply and reverting parts of code

## Query Semantics

Explain `Querier` here as well
