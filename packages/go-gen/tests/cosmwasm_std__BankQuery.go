type SupplyQuery struct {
	Denom string `json:"denom"`
}

type BalanceQuery struct {
	Address string `json:"address"`
	Denom   string `json:"denom"`
}

type AllBalancesQuery struct {
	Address string `json:"address"`
}

type DenomMetadataQuery struct {
	Denom string `json:"denom"`
}

type AllDenomMetadataQuery struct {
	// Pagination is an optional argument.
	// Default pagination will be used if this is omitted
	Pagination *PageRequest `json:"pagination,omitempty"`
}

type BankQuery struct {
	Supply           *SupplyQuery           `json:"supply,omitempty"`
	Balance          *BalanceQuery          `json:"balance,omitempty"`
	AllBalances      *AllBalancesQuery      `json:"all_balances,omitempty"`
	DenomMetadata    *DenomMetadataQuery    `json:"denom_metadata,omitempty"`
	AllDenomMetadata *AllDenomMetadataQuery `json:"all_denom_metadata,omitempty"`
}

// Simplified version of the cosmos-sdk PageRequest type
type PageRequest struct {
	// Key is a value returned in PageResponse.next_key to begin
	// querying the next page most efficiently. Only one of offset or key
	// should be set.
	Key []byte `json:"key"`
	// Limit is the total number of results to be returned in the result page.
	// If left empty it will default to a value to be set by each app.
	Limit uint32 `json:"limit"`
	// Reverse is set to true if results are to be returned in the descending order.
	Reverse bool `json:"reverse"`
}