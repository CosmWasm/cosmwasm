
type DelegatorWithdrawAddressQuery struct {
	DelegatorAddress string `json:"delegator_address"`
}

type DistributionQuery struct {
	DelegatorWithdrawAddress *DelegatorWithdrawAddressQuery `json:"delegator_withdraw_address,omitempty"`
}