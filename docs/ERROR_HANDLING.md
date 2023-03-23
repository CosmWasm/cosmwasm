# Error handling for various entry points

In this document we discuss how different types of errors during contract
execution are handled by wasmd and the blockchain.

## Two levels of errors

When cosmwasm-vm executes a contract, the caller receives a nested result type:
`VmResult<ContractResult<R>>` with some success response `R`. The outer
`VmResult` is created by the host environment and the inner `ContractResult` is
created inside of the contract. Most application specific error should go into
`ContractResult` errors. This is what happens when you use `?` inside of your
contract implementations. The `VmResult`
[error cases](https://github.com/CosmWasm/cosmwasm/blob/v1.2.3/packages/vm/src/errors/vm_error.rs#L11-L148)
include e.g.

- Caching errors such as a missing Wasm file or corrupted module
- Serialization problems in the contract-host communication
- Panics from panic handler in contract
- Errors in crypto API calls
- Out of gas
- Unreachable statements in the Wasm bytecode

## Error handling

In wasmvm those two error types are merged into one and handled as one thing in
the caller (wasmd):

- [Instantiate](https://github.com/CosmWasm/wasmvm/blob/v1.2.0/lib.go#L144-L151)
- [Execute](https://github.com/CosmWasm/wasmvm/blob/v1.2.0/lib.go#L192-L199)
- [Migtate](https://github.com/CosmWasm/wasmvm/blob/v1.2.0/lib.go#L275-L282)
- [Sudo](https://github.com/CosmWasm/wasmvm/blob/v1.2.0/lib.go#L318-L325)
- [Reply](https://github.com/CosmWasm/wasmvm/blob/v1.2.0/lib.go#L363-L370)
- [IBCChannelOpen](https://github.com/CosmWasm/wasmvm/blob/v1.2.0/lib.go#L406-L413)
- [IBCChannelConnect](https://github.com/CosmWasm/wasmvm/blob/v1.2.0/lib.go#L449-L456)
- [IBCChannelClose](https://github.com/CosmWasm/wasmvm/blob/v1.2.0/lib.go#L492-L499)
- [IBCPacketAck](https://github.com/CosmWasm/wasmvm/blob/v1.2.0/lib.go#L576-L583)
- [IBCPacketTimeout](https://github.com/CosmWasm/wasmvm/blob/v1.2.0/lib.go#L620-L627)

However, there is one exception:

- [IBCPacketReceive](https://github.com/CosmWasm/wasmvm/blob/v1.2.0/lib.go#L535-L539)

Instead of returning only the contents of the `Ok` case, the whole
`IBCReceiveResult` is returned. This allows the caller to handle the two layers
of errors differently.

As pointed out by our auditors from Oak Security, this
[is inconsistent](https://github.com/CosmWasm/wasmvm/issues/398). Historically
merging the two error types was the desired behaviour. When `IBCPacketReceive`
came in, we needed the differentiation to be available in wasmd, which is why
the API is different than the others. Ideally we always return the contract
Result and let wasmd handle it.

## Handing ibc_packet_receive errors

For wasmd before 0.32, contract errors and VM errors were handled the same. They
got the special treatment of reverting state changes, writing an error
acknowledgement but don't let the transaction fail.

For wasmd >= 0.32, the special treatment only applies to contract errors. VM
errors in `IBCPacketReceive` let the transaction fail just like the `Execute`
case would. This has two major implications:

1. Application specific errors (especially those which can be triggered by
   untrusted users) should create contract errors and no panics. This ensures
   that error acknowledgements are written and relayer transactions don't fail.
2. Using panics allow the contract developer to make the transaction fail
   without writing an acknowledgement. This can be handy e.g. for allowlisting
   relayer addresses.

The following table shows the new handling logic.

| Entry point           | Contract error                                     | VM error                                      |
| --------------------- | -------------------------------------------------- | --------------------------------------------- |
| `instantiate`         | ⏮️ state reverted<br>❌ tx fails                   | ⏮️ state reverted<br>❌ tx fails              |
| `execute`             | ⏮️ state reverted<br>❌ tx fails                   | ⏮️ state reverted<br>❌ tx fails              |
| `migrate`             | ⏮️ state reverted<br>❌ tx fails                   | ⏮️ state reverted<br>❌ tx fails              |
| `sudo`                | ⏮️ state reverted<br>❌ tx fails                   | ⏮️ state reverted<br>❌ tx fails              |
| `reply`               | ⏮️ state reverted<br>❔ depends on `reply_on`      | ⏮️ state reverted<br>❔ depends on `reply_on` |
| `ibc_channel_open`    | ⏮️ state reverted<br>❌ tx fails                   | ⏮️ state reverted<br>❌ tx fails              |
| `ibc_channel_connect` | ⏮️ state reverted<br>❌ tx fails                   | ⏮️ state reverted<br>❌ tx fails              |
| `ibc_channel_close`   | ⏮️ state reverted<br>❌ tx fails                   | ⏮️ state reverted<br>❌ tx fails              |
| `ibc_packet_receive`  | ⏮️ state reverted<br>✅ tx succeeds with error ack | ⏮️ state reverted<br>❌ tx fails              |
| `ibc_packet_ack`      | ⏮️ state reverted<br>❌ tx fails                   | ⏮️ state reverted<br>❌ tx fails              |
| `ibc_packet_timeout`  | ⏮️ state reverted<br>❌ tx fails                   | ⏮️ state reverted<br>❌ tx fails              |
