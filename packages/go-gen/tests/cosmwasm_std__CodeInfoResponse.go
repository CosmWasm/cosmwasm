type CodeInfoResponse struct {
	Checksum Checksum `json:"checksum"` // before wasmvm 2.0.0, this was `omitempty` (https://github.com/CosmWasm/wasmvm/issues/471)
	CodeID   uint64   `json:"code_id"`
	Creator  string   `json:"creator"`
}
