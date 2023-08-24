type DenomMetadataResponse struct {
	Metadata DenomMetadata `json:"metadata"`
}

// Replicating the cosmos-sdk bank module Metadata type
type DenomMetadata struct {
	// Base represents the base denom (should be the DenomUnit with exponent = 0).
	Base string `json:"base"`
	// DenomUnits represents the list of DenomUnits for a given coin
	DenomUnits  []DenomUnit `json:"denom_units"`
	Description string      `json:"description"`
	// Display indicates the suggested denom that should be
	// displayed in clients.
	Display string `json:"display"`
	// Name defines the name of the token (eg: Cosmos Atom)
	//
	// Since: cosmos-sdk 0.43
	Name string `json:"name"`
	// Symbol is the token symbol usually shown on exchanges (eg: ATOM). This can
	// be the same as the display.
	//
	// Since: cosmos-sdk 0.43
	Symbol string `json:"symbol"`
	// URI to a document (on or off-chain) that contains additional information. Optional.
	//
	// Since: cosmos-sdk 0.46
	URI string `json:"uri"`
	// URIHash is a sha256 hash of a document pointed by URI. It's used to verify that
	// the document didn't change. Optional.
	//
	// Since: cosmos-sdk 0.46
	URIHash string `json:"uri_hash"`
}

// Replicating the cosmos-sdk bank module DenomUnit type
type DenomUnit struct {
	// Aliases is a list of string aliases for the given denom
	Aliases []string `json:"aliases"`
	// Denom represents the string name of the given denom unit (e.g uatom).
	Denom string `json:"denom"`
	// Exponent represents power of 10 exponent that one must
	// raise the base_denom to in order to equal the given DenomUnit's denom
	// 1 denom = 10^exponent base_denom
	// (e.g. with a base_denom of uatom, one can create a DenomUnit of 'atom' with
	// exponent = 6, thus: 1 atom = 10^6 uatom).
	Exponent uint32 `json:"exponent"`
}