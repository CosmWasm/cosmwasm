type BondedDenomQuery struct { // does not exist in wasmvm, but is an anonymous struct instead
}

type AllDelegationsQuery struct {
	Delegator string `json:"delegator"`
}

type DelegationQuery struct {
	Delegator string `json:"delegator"`
	Validator string `json:"validator"`
}

type AllValidatorsQuery struct {
}

type ValidatorQuery struct {
	/// Address is the validator's address (e.g. cosmosvaloper1...)
	Address string `json:"address"`
}

type StakingQuery struct {
	BondedDenom    *BondedDenomQuery    `json:"bonded_denom,omitempty"`
	AllDelegations *AllDelegationsQuery `json:"all_delegations,omitempty"`
	Delegation     *DelegationQuery     `json:"delegation,omitempty"`
	AllValidators  *AllValidatorsQuery  `json:"all_validators,omitempty"`
	Validator      *ValidatorQuery      `json:"validator,omitempty"`
}