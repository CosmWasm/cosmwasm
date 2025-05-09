
// SmartQuery response is raw bytes ([]byte)
type SmartQuery struct {
	// Bech32 encoded sdk.AccAddress of the contract
	ContractAddr string `json:"contract_addr"`
	Msg          []byte `json:"msg"`
}

// RawQuery response is raw bytes ([]byte)
type RawQuery struct {
	// Bech32 encoded sdk.AccAddress of the contract
	ContractAddr string `json:"contract_addr"`
	Key          []byte `json:"key"`
}

type ContractInfoQuery struct {
	// Bech32 encoded sdk.AccAddress of the contract
	ContractAddr string `json:"contract_addr"`
}

type CodeInfoQuery struct {
	CodeID uint64 `json:"code_id"`
}

type RawRangeQuery struct {
	// The address of the contract to query
	ContractAddr string `json:"contract_addr"`
	// Exclusive end bound. This is the key after the last key you would like to get data for.
	End *[]byte `json:"end,omitempty"`
	// Maximum number of elements to return.
	//
	// Make sure to set a reasonable limit to avoid running out of memory or into the deserialization limits of the VM. Also keep in mind that these limitations depend on the full JSON size of the response type.
	Limit uint16 `json:"limit"`
	// The order in which you want to receive the key-value pairs.
	Order string `json:"order"`
	// Inclusive start bound. This is the first key you would like to get data for.
	//
	// If `start` is lexicographically greater than or equal to `end`, an empty range is described, mo matter of the order.
	Start *[]byte `json:"start,omitempty"`
}

type WasmQuery struct {
	Smart        *SmartQuery        `json:"smart,omitempty"`
	Raw          *RawQuery          `json:"raw,omitempty"`
	ContractInfo *ContractInfoQuery `json:"contract_info,omitempty"`
	CodeInfo     *CodeInfoQuery     `json:"code_info,omitempty"`
	RawRange     *RawRangeQuery     `json:"raw_range,omitempty"`
}