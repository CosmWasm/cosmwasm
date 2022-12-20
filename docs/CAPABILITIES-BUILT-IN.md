# Built-in capabilities

Since capabilities can be created between contract and environment, we don't
know them all in the VM. This is a list of all built-in capabilities, but chains
might define others.

- `iterator` is for storage backends that allow range queries. Not all types of
  databases do that. There are trees that don't allow it and Secret Network does
  not support iterators for other technical reasons.
- `stargate` is for messages and queries that came with the Cosmos SDK upgrade
  "Stargate". It primarily includes protobuf messages and IBC support.
- `staking` is for chains with the Cosmos SDK staking module. There are Cosmos
  chains that don't use this (e.g. Tgrade).
- `cosmwasm_1_1` enables the `BankQuery::Supply` query. Only chains running
  CosmWasm `1.1.0` or higher support this.
- `cosmwasm_1_2` enables the `GovMsg::VoteWeighted` and `WasmMsg::Instantiate2`
  messages. Only chains running CosmWasm `1.2.0` or higher support this.
