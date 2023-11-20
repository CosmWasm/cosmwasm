type TransferMsg struct {
	Amount    Coin       `json:"amount"`
	ChannelID string     `json:"channel_id"`
	Memo      string     `json:"memo,omitempty"` // this is not yet in wasmvm, but will be soon
	Timeout   IBCTimeout `json:"timeout"`
	ToAddress string     `json:"to_address"`
}
type SendPacketMsg struct {
	ChannelID string     `json:"channel_id"`
	Data      []byte     `json:"data"`
	Timeout   IBCTimeout `json:"timeout"`
}
type CloseChannelMsg struct {
	ChannelID string `json:"channel_id"`
}

type IBCMsg struct {
	Transfer     *TransferMsg     `json:"transfer,omitempty"`
	SendPacket   *SendPacketMsg   `json:"send_packet,omitempty"`
	CloseChannel *CloseChannelMsg `json:"close_channel,omitempty"`
}

// Coin is a string representation of the sdk.Coin type (more portable than sdk.Int)
type Coin struct {
	Amount string `json:"amount"` // string encoing of decimal value, eg. "12.3456"
	Denom  string `json:"denom"`  // type, eg. "ATOM"
}

// IBCTimeout is the timeout for an IBC packet. At least one of block and timestamp is required.
type IBCTimeout struct {
	Block *IBCTimeoutBlock `json:"block,omitempty"` // in wasmvm, this does not have "omitempty"
	// Nanoseconds since UNIX epoch
	Timestamp *Uint64 `json:"timestamp,omitempty"`
}

// IBCTimeoutBlock Height is a monotonically increasing data type
// that can be compared against another Height for the purposes of updating and
// freezing clients.
// Ordering is (revision_number, timeout_height)
type IBCTimeoutBlock struct {
	// block height after which the packet times out.
	// the height within the given revision
	Height uint64 `json:"height"`
	// the version that the client is currently on
	// (eg. after reseting the chain this could increment 1 as height drops to 0)
	Revision uint64 `json:"revision"`
}
