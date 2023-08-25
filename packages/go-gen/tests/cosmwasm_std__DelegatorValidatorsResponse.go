// See <https://github.com/cosmos/cosmos-sdk/blob/b0acf60e6c39f7ab023841841fc0b751a12c13ff/proto/cosmos/distribution/v1beta1/query.proto#L212-L220>
type DelegatorValidatorsResponse struct {
	Validators []string `json:"validators"`
}