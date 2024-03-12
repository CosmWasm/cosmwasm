// AllDelegationsResponse is the expected response to AllDelegationsQuery
type AllDelegationsResponse struct {
	Delegations Array[Delegation] `json:"delegations"`
}

// Coin is a string representation of the sdk.Coin type (more portable than sdk.Int)
type Coin struct {
	Amount string `json:"amount"` // string encoing of decimal value, eg. "12.3456"
	Denom  string `json:"denom"`  // type, eg. "ATOM"
}

type Delegation struct {
	Amount    Coin   `json:"amount"`
	Delegator string `json:"delegator"`
	Validator string `json:"validator"`
}
