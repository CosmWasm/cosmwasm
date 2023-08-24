// AllBalancesResponse is the expected response to AllBalancesQuery
type AllBalancesResponse struct {
	Amount []Coin `json:"amount"` // in wasmvm, there is an alias for `[]Coin`
}

// Coin is a string representation of the sdk.Coin type (more portable than sdk.Int)
type Coin struct {
	Amount string `json:"amount"` // string encoing of decimal value, eg. "12.3456"
	Denom  string `json:"denom"`  // type, eg. "ATOM"
}
