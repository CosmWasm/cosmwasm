type ContractInfoResponse struct {
	// admin who can run migrations (if any)
	Admin  string `json:"admin,omitempty"`
	CodeID uint64 `json:"code_id"`
	// address that instantiated this contract
	Creator string `json:"creator"`
	// set if this contract has bound an IBC port
	IBCPort string `json:"ibc_port,omitempty"`
	// set if this contract has bound an IBCv2 port
	Ibcv2Port string `json:"ibcv2_port,omitempty"`
	// if set, the contract is pinned to the cache, and thus uses less gas when called
	Pinned bool `json:"pinned"`
}