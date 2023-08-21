// AllValidatorsResponse is the expected response to AllValidatorsQuery
type AllValidatorsResponse struct {
	Validators []Validator `json:"validators"` // in wasmvm, there is an alias for `[]Validator`
}

type Validator struct {
	Address string `json:"address"`
	// decimal string, eg "0.02"
	Commission string `json:"commission"`
	// decimal string, eg "0.02"
	MaxChangeRate string `json:"max_change_rate"`
	// decimal string, eg "0.02"
	MaxCommission string `json:"max_commission"`
}