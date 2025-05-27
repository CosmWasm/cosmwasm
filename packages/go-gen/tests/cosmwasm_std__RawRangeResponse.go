type RawRangeResponse struct {
	// The key-value pairs
	Data Array[Array[[]byte]] `json:"data"`
	// `None` if there are no more key-value pairs within the given key range.
	NextKey []byte `json:"next_key"`
}
