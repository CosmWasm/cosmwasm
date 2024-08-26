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
type WriteAcknowledgementMsg struct {
	// The acknowledgement to send back
	Ack IBCAcknowledgement `json:"ack"`
	// Existing channel where the packet was received
	ChannelID string `json:"channel_id"`
	// Sequence number of the packet that was received
	PacketSequence uint64 `json:"packet_sequence"`
}
type CloseChannelMsg struct {
	ChannelID string `json:"channel_id"`
}
type PayPacketFeeMsg struct {
	// The channel id on the chain where the packet is sent from (this chain).
	ChannelID string `json:"channel_id"`
	Fee       IBCFee `json:"fee"`
	// The port id on the chain where the packet is sent from (this chain).
	PortID string `json:"port_id"`
	// Allowlist of relayer addresses that can receive the fee. This is currently not implemented and *must* be empty.
	Relayers Array[string] `json:"relayers"`
}
type PayPacketFeeAsyncMsg struct {
	// The channel id on the chain where the packet is sent from (this chain).
	ChannelID string `json:"channel_id"`
	Fee       IBCFee `json:"fee"`
	// The port id on the chain where the packet is sent from (this chain).
	PortID string `json:"port_id"`
	// Allowlist of relayer addresses that can receive the fee. This is currently not implemented and *must* be empty.
	Relayers Array[string] `json:"relayers"`
	// The sequence number of the packet that should be incentivized.
	Sequence uint64 `json:"sequence"`
}

type IBCMsg struct {
	Transfer             *TransferMsg             `json:"transfer,omitempty"`
	SendPacket           *SendPacketMsg           `json:"send_packet,omitempty"`
	WriteAcknowledgement *WriteAcknowledgementMsg `json:"write_acknowledgement,omitempty"`
	CloseChannel         *CloseChannelMsg         `json:"close_channel,omitempty"`
	PayPacketFee         *PayPacketFeeMsg         `json:"pay_packet_fee,omitempty"`
	PayPacketFeeAsync    *PayPacketFeeAsyncMsg    `json:"pay_packet_fee_async,omitempty"`
}

// Coin is a string representation of the sdk.Coin type (more portable than sdk.Int)
type Coin struct {
	Amount string `json:"amount"` // string encoing of decimal value, eg. "12.3456"
	Denom  string `json:"denom"`  // type, eg. "ATOM"
}

type IBCAcknowledgement struct {
	Data []byte `json:"data"`
}
type IBCFee struct {
	AckFee     Array[Coin] `json:"ack_fee"`
	ReceiveFee Array[Coin] `json:"receive_fee"`
	TimeoutFee Array[Coin] `json:"timeout_fee"`
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
