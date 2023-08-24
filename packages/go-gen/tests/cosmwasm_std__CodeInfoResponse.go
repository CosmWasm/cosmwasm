type CodeInfoResponse struct {
	Checksum Checksum `json:"checksum,omitempty"`
	CodeID   uint64   `json:"code_id"`
	Creator  string   `json:"creator"`
}
