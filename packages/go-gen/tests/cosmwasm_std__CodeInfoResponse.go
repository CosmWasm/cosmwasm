type CodeInfoResponse struct {
	Checksum Checksum `json:"checksum"` // in wasmvm, this is `omitempty`
	CodeID   uint64   `json:"code_id"`
	Creator  string   `json:"creator"`
}
