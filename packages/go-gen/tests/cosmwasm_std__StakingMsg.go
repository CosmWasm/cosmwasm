type DelegateMsg struct {
	Amount    Coin   `json:"amount"`
	Validator string `json:"validator"`
}
type UndelegateMsg struct {
	Amount    Coin   `json:"amount"`
	Validator string `json:"validator"`
}
type RedelegateMsg struct {
	Amount       Coin   `json:"amount"`
	DstValidator string `json:"dst_validator"`
	SrcValidator string `json:"src_validator"`
}

type StakingMsg struct {
	Delegate   *DelegateMsg   `json:"delegate,omitempty"`
	Undelegate *UndelegateMsg `json:"undelegate,omitempty"`
	Redelegate *RedelegateMsg `json:"redelegate,omitempty"`
}

// Coin is a string representation of the sdk.Coin type (more portable than sdk.Int)
type Coin struct {
	Amount string `json:"amount"` // string encoing of decimal value, eg. "12.3456"
	Denom  string `json:"denom"`  // type, eg. "ATOM"
}
