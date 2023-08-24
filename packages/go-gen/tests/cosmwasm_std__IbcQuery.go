type PortIDQuery struct {
}

// ListChannelsQuery is an IBCQuery that lists all channels that are bound to a given port.
// If `PortID` is unset, this list all channels bound to the contract's port.
// Returns a `ListChannelsResponse`.
// This is the counterpart of [IbcQuery::ListChannels](https://github.com/CosmWasm/cosmwasm/blob/v0.14.0-beta1/packages/std/src/ibc.rs#L70-L73).
type ListChannelsQuery struct {
	// optional argument
	PortID string `json:"port_id,omitempty"`
}

type ChannelQuery struct {
	ChannelID string `json:"channel_id"`
	// optional argument
	PortID string `json:"port_id,omitempty"`
}

// IBCQuery defines a query request from the contract into the chain.
// This is the counterpart of [IbcQuery](https://github.com/CosmWasm/cosmwasm/blob/v0.14.0-beta1/packages/std/src/ibc.rs#L61-L83).
type IBCQuery struct {
	PortID       *PortIDQuery       `json:"port_id,omitempty"`
	ListChannels *ListChannelsQuery `json:"list_channels,omitempty"`
	Channel      *ChannelQuery      `json:"channel,omitempty"`
}
