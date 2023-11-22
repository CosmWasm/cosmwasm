// ExecuteMsg is used to call another defined contract on this chain.
// The calling contract requires the callee to be defined beforehand,
// and the address should have been defined in initialization.
// And we assume the developer tested the ABIs and coded them together.
//
// Since a contract is immutable once it is deployed, we don't need to transform this.
// If it was properly coded and worked once, it will continue to work throughout upgrades.
type ExecuteMsg struct {
	// ContractAddr is the sdk.AccAddress of the contract, which uniquely defines
	// the contract ID and instance ID. The sdk module should maintain a reverse lookup table.
	ContractAddr string `json:"contract_addr"`
	// Send is an optional amount of coins this contract sends to the called contract
	Funds []Coin `json:"funds"`
	// Msg is assumed to be a json-encoded message, which will be passed directly
	// as `userMsg` when calling `Handle` on the above-defined contract
	Msg []byte `json:"msg"`
}

// InstantiateMsg will create a new contract instance from a previously uploaded CodeID.
// This allows one contract to spawn "sub-contracts".
type InstantiateMsg struct {
	// Admin (optional) may be set here to allow future migrations from this address
	Admin string `json:"admin,omitempty"`
	// CodeID is the reference to the wasm byte code as used by the Cosmos-SDK
	CodeID uint64 `json:"code_id"`
	// Send is an optional amount of coins this contract sends to the called contract
	Funds []Coin `json:"funds"`
	// Label is optional metadata to be stored with a contract instance.
	Label string `json:"label"`
	// Msg is assumed to be a json-encoded message, which will be passed directly
	// as `userMsg` when calling `Instantiate` on a new contract with the above-defined CodeID
	Msg []byte `json:"msg"`
}

// Instantiate2Msg will create a new contract instance from a previously uploaded CodeID
// using the predictable address derivation.
type Instantiate2Msg struct {
	// Admin (optional) may be set here to allow future migrations from this address
	Admin string `json:"admin,omitempty"`
	// CodeID is the reference to the wasm byte code as used by the Cosmos-SDK
	CodeID uint64 `json:"code_id"`
	// Send is an optional amount of coins this contract sends to the called contract
	Funds []Coin `json:"funds"`
	// Label is optional metadata to be stored with a contract instance.
	Label string `json:"label"`
	// Msg is assumed to be a json-encoded message, which will be passed directly
	// as `userMsg` when calling `Instantiate` on a new contract with the above-defined CodeID
	Msg  []byte `json:"msg"`
	Salt []byte `json:"salt"`
}

// MigrateMsg will migrate an existing contract from it's current wasm code (logic)
// to another previously uploaded wasm code. It requires the calling contract to be
// listed as "admin" of the contract to be migrated.
type MigrateMsg struct {
	// ContractAddr is the sdk.AccAddress of the target contract, to migrate.
	ContractAddr string `json:"contract_addr"`
	// Msg is assumed to be a json-encoded message, which will be passed directly
	// as `userMsg` when calling `Migrate` on the above-defined contract
	Msg []byte `json:"msg"`
	// NewCodeID is the reference to the wasm byte code for the new logic to migrate to
	NewCodeID uint64 `json:"new_code_id"`
}

// UpdateAdminMsg is the Go counterpart of WasmMsg::UpdateAdmin
// (https://github.com/CosmWasm/cosmwasm/blob/v0.14.0-beta5/packages/std/src/results/cosmos_msg.rs#L158-L160).
type UpdateAdminMsg struct {
	// Admin is the sdk.AccAddress of the new admin.
	Admin string `json:"admin"`
	// ContractAddr is the sdk.AccAddress of the target contract.
	ContractAddr string `json:"contract_addr"`
}

// ClearAdminMsg is the Go counterpart of WasmMsg::ClearAdmin
// (https://github.com/CosmWasm/cosmwasm/blob/v0.14.0-beta5/packages/std/src/results/cosmos_msg.rs#L158-L160).
type ClearAdminMsg struct {
	// ContractAddr is the sdk.AccAddress of the target contract.
	ContractAddr string `json:"contract_addr"`
}

// The message types of the wasm module.
//
// See https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto
type WasmMsg struct {
	// Dispatches a call to another contract at a known address (with known ABI).
	//
	// This is translated to a [MsgExecuteContract](https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto#L68-L78). `sender` is automatically filled with the current contract's address.
	Execute *ExecuteMsg `json:"execute,omitempty"`
	// Instantiates a new contracts from previously uploaded Wasm code.
	//
	// The contract address is non-predictable. But it is guaranteed that when emitting the same Instantiate message multiple times, multiple instances on different addresses will be generated. See also Instantiate2.
	//
	// This is translated to a [MsgInstantiateContract](https://github.com/CosmWasm/wasmd/blob/v0.29.2/proto/cosmwasm/wasm/v1/tx.proto#L53-L71). `sender` is automatically filled with the current contract's address.
	Instantiate *InstantiateMsg `json:"instantiate,omitempty"`
	// Instantiates a new contracts from previously uploaded Wasm code using a predictable address derivation algorithm implemented in [`cosmwasm_std::instantiate2_address`].
	//
	// This is translated to a [MsgInstantiateContract2](https://github.com/CosmWasm/wasmd/blob/v0.29.2/proto/cosmwasm/wasm/v1/tx.proto#L73-L96). `sender` is automatically filled with the current contract's address. `fix_msg` is automatically set to false.
	Instantiate2 *Instantiate2Msg `json:"instantiate2,omitempty"`
	// Migrates a given contracts to use new wasm code. Passes a MigrateMsg to allow us to customize behavior.
	//
	// Only the contract admin (as defined in wasmd), if any, is able to make this call.
	//
	// This is translated to a [MsgMigrateContract](https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto#L86-L96). `sender` is automatically filled with the current contract's address.
	Migrate *MigrateMsg `json:"migrate,omitempty"`
	// Sets a new admin (for migrate) on the given contract. Fails if this contract is not currently admin of the target contract.
	UpdateAdmin *UpdateAdminMsg `json:"update_admin,omitempty"`
	// Clears the admin on the given contract, so no more migration possible. Fails if this contract is not currently admin of the target contract.
	ClearAdmin *ClearAdminMsg `json:"clear_admin,omitempty"`
}

// Coin is a string representation of the sdk.Coin type (more portable than sdk.Int)
type Coin struct {
	Amount string `json:"amount"` // string encoing of decimal value, eg. "12.3456"
	Denom  string `json:"denom"`  // type, eg. "ATOM"
}
