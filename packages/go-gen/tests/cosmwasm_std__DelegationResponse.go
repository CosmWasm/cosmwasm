// DelegationResponse is the expected response to DelegationsQuery
type DelegationResponse struct {
	Delegation *FullDelegation `json:"delegation,omitempty"`
}

// Coin is a string representation of the sdk.Coin type (more portable than sdk.Int)
type Coin struct {
	Amount string `json:"amount"` // string encoing of decimal value, eg. "12.3456"
	Denom  string `json:"denom"`  // type, eg. "ATOM"
}

type FullDelegation struct {
	AccumulatedRewards []Coin `json:"accumulated_rewards"` // in wasmvm, there is an alias for `[]Coin`
	Amount             Coin   `json:"amount"`
	CanRedelegate      Coin   `json:"can_redelegate"`
	Delegator          string `json:"delegator"`
	Validator          string `json:"validator"`
}