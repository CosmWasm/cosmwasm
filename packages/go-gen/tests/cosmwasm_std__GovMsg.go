type VoteMsg struct {
	// Option is the vote option.
	//
	// This used to be called "vote", but was changed for consistency with Cosmos SDK.
	// The old name is still supported for backwards compatibility.
	Option     voteOption `json:"option"`
	ProposalID uint64     `json:"proposal_id"` // in wasmvm, this is `ProposalId`
}

type VoteWeightedMsg struct {
	Options    Array[WeightedVoteOption] `json:"options"`     // in wasmvm, this has type `[]WeightedVoteOption`
	ProposalID uint64                    `json:"proposal_id"` // in wasmvm, this is `ProposalId`
}

type GovMsg struct {
	// This maps directly to [MsgVote](https://github.com/cosmos/cosmos-sdk/blob/v0.42.5/proto/cosmos/gov/v1beta1/tx.proto#L46-L56) in the Cosmos SDK with voter set to the contract address.
	Vote *VoteMsg `json:"vote,omitempty"`
	/// This maps directly to [MsgVoteWeighted](https://github.com/cosmos/cosmos-sdk/blob/v0.45.8/proto/cosmos/gov/v1beta1/tx.proto#L66-L78) in the Cosmos SDK with voter set to the contract address.
	VoteWeighted *VoteWeightedMsg `json:"vote_weighted,omitempty"`
}

type WeightedVoteOption struct {
	Option voteOption `json:"option"`
	// Weight is a Decimal string, e.g. "0.25" for 25%
	Weight string `json:"weight"`
}
