// See <https://github.com/cosmos/cosmos-sdk/blob/c74e2887b0b73e81d48c2f33e6b1020090089ee0/proto/cosmos/distribution/v1beta1/query.proto#L169-L178>
type DelegationRewardsResponse struct {
	Rewards Array[DecCoin] `json:"rewards"` // in wasmvm, this has type `[]DecCoin`
}

// A coin type with decimal amount. Modeled after the Cosmos SDK's [DecCoin](https://github.com/cosmos/cosmos-sdk/blob/c74e2887b0b73e81d48c2f33e6b1020090089ee0/proto/cosmos/base/v1beta1/coin.proto#L32-L41) type
type DecCoin struct {
	Amount string `json:"amount"`
	Denom  string `json:"denom"`
}
