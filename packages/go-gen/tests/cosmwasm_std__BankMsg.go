// SendMsg contains instructions for a Cosmos-SDK/SendMsg
// It has a fixed interface here and should be converted into the proper SDK format before dispatching
type SendMsg struct {
	Amount    []Coin `json:"amount"`
	ToAddress string `json:"to_address"`
}

// BurnMsg will burn the given coins from the contract's account.
// There is no Cosmos SDK message that performs this, but it can be done by calling the bank keeper.
// Important if a contract controls significant token supply that must be retired.
type BurnMsg struct {
	Amount []Coin `json:"amount"`
}

type BankMsg struct {
	Send *SendMsg `json:"send,omitempty"`
	Burn *BurnMsg `json:"burn,omitempty"`
}

// Coin is a string representation of the sdk.Coin type (more portable than sdk.Int)
type Coin struct {
	Amount string `json:"amount"` // string encoing of decimal value, eg. "12.3456"
	Denom  string `json:"denom"`  // type, eg. "ATOM"
}
