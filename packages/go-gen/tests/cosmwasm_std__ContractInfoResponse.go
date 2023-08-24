type ContractInfoResponse struct {
	// Set to the admin who can migrate contract, if any
	Admin   string `json:"admin,omitempty"`
	CodeID  uint64 `json:"code_id"`
	Creator string `json:"creator"`
	// Set if the contract is IBC enabled
	IBCPort string `json:"ibc_port,omitempty"`
	Pinned  bool   `json:"pinned"`
}