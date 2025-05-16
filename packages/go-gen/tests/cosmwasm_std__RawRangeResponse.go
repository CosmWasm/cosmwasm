type RawRangeResponse struct {
	// The key-value pairs
	Data Array[RawRangeEntry] `json:"data"`
	// `None` if there are no more key-value pairs within the given key range.
	NextKey *[]byte `json:"next_key,omitempty"`
}
type RawRangeEntry struct {
	K []byte `json:"k"`
	V []byte `json:"v"`
}