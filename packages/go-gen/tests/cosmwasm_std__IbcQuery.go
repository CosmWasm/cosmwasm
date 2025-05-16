type PortIDQuery struct {
}

type ChannelQuery struct {
	ChannelID string `json:"channel_id"`
	// optional argument
	PortID string `json:"port_id,omitempty"`
}

// IBCQuery defines a query request from the contract into the chain.
// This is the counterpart of [IbcQuery](https://github.com/CosmWasm/cosmwasm/blob/v0.14.0-beta1/packages/std/src/ibc.rs#L61-L83).
type IBCQuery struct {
	PortID  *PortIDQuery  `json:"port_id,omitempty"`
	Channel *ChannelQuery `json:"channel,omitempty"`
}
