type DelegatorWithdrawAddressQuery struct {
	DelegatorAddress string `json:"delegator_address"`
}
type DelegationRewardsQuery struct {
	DelegatorAddress string `json:"delegator_address"`
	ValidatorAddress string `json:"validator_address"`
}
type DelegationTotalRewardsQuery struct {
	DelegatorAddress string `json:"delegator_address"`
}
type DelegatorValidatorsQuery struct {
	DelegatorAddress string `json:"delegator_address"`
}

type DistributionQuery struct {
	// See <https://github.com/cosmos/cosmos-sdk/blob/c74e2887b0b73e81d48c2f33e6b1020090089ee0/proto/cosmos/distribution/v1beta1/query.proto#L222-L230>
	DelegatorWithdrawAddress *DelegatorWithdrawAddressQuery `json:"delegator_withdraw_address,omitempty"`
	// See <https://github.com/cosmos/cosmos-sdk/blob/c74e2887b0b73e81d48c2f33e6b1020090089ee0/proto/cosmos/distribution/v1beta1/query.proto#L157-L167>
	DelegationRewards *DelegationRewardsQuery `json:"delegation_rewards,omitempty"`
	// See <https://github.com/cosmos/cosmos-sdk/blob/c74e2887b0b73e81d48c2f33e6b1020090089ee0/proto/cosmos/distribution/v1beta1/query.proto#L180-L187>
	DelegationTotalRewards *DelegationTotalRewardsQuery `json:"delegation_total_rewards,omitempty"`
	// See <https://github.com/cosmos/cosmos-sdk/blob/b0acf60e6c39f7ab023841841fc0b751a12c13ff/proto/cosmos/distribution/v1beta1/query.proto#L202-L210>
	DelegatorValidators *DelegatorValidatorsQuery `json:"delegator_validators,omitempty"`
}