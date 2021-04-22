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

Result<Response>

### Dispatching Messages

Fire and forget

### Submessages

Reply and reverting parts of code

## Query Semantics

Explain `Querier` here as well
