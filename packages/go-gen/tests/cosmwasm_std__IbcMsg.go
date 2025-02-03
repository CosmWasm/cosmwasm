type TransferMsg struct {
	// packet data only supports one coin https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/ibc/applications/transfer/v1/transfer.proto#L11-L20
	Amount Coin `json:"amount"`
	// existing channel to send the tokens over
	ChannelID string `json:"channel_id"`
	// An optional memo. See the blog post ["Moving Beyond Simple Token Transfers"](https://medium.com/the-interchain-foundation/moving-beyond-simple-token-transfers-d42b2b1dc29b) for more information.
	//
	// There is no difference between setting this to `None` or an empty string.
	//
	// This field is only supported on chains with CosmWasm >= 2.0 and silently ignored on older chains. If you need support for both 1.x and 2.x chain with the same codebase, it is recommended to use `CosmosMsg::Stargate` with a custom MsgTransfer protobuf encoder instead.
	Memo string `json:"memo,omitempty"`
	// when packet times out, measured on remote chain
	Timeout IBCTimeout `json:"timeout"`
	// address on the remote chain to receive these tokens
	ToAddress string `json:"to_address"`
}
type TransferV2Msg struct {
	// existing channel to send the tokens over
	ChannelID  string     `json:"channel_id"`
	Forwarding Array[Hop] `json:"forwarding"`
	// An optional memo. See the blog post ["Moving Beyond Simple Token Transfers"](https://medium.com/the-interchain-foundation/moving-beyond-simple-token-transfers-d42b2b1dc29b) for more information.
	//
	// There is no difference between setting this to `None` or an empty string.
	Memo string `json:"memo,omitempty"`
	// when packet times out, measured on remote chain
	Timeout IBCTimeout `json:"timeout"`
	// address on the remote chain to receive these tokens
	ToAddress string `json:"to_address"`
	// packet data only supports one coin https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/ibc/applications/transfer/v1/transfer.proto#L11-L20
	Tokens Array[Coin] `json:"tokens"`
}
type SendPacketMsg struct {
	ChannelID string `json:"channel_id"`
	Data      []byte `json:"data"`
	// when packet times out, measured on remote chain
	Timeout IBCTimeout `json:"timeout"`
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
	// Allowlist of relayer addresses that can receive the fee. An empty list means that any relayer can receive the fee.
	//
	// This is currently not implemented and *must* be empty.
	Relayers Array[string] `json:"relayers"`
}
type PayPacketFeeAsyncMsg struct {
	// The channel id on the chain where the packet is sent from (this chain).
	ChannelID string `json:"channel_id"`
	Fee       IBCFee `json:"fee"`
	// The port id on the chain where the packet is sent from (this chain).
	PortID string `json:"port_id"`
	// Allowlist of relayer addresses that can receive the fee. An empty list means that any relayer can receive the fee.
	//
	// This is currently not implemented and *must* be empty.
	Relayers Array[string] `json:"relayers"`
	// The sequence number of the packet that should be incentivized.
	Sequence uint64 `json:"sequence"`
}

// These are messages in the IBC lifecycle. Only usable by IBC-enabled contracts (contracts that directly speak the IBC protocol via 6 entry points)
type IBCMsg struct {
	// Sends bank tokens owned by the contract to the given address on another chain. The channel must already be established between the ibctransfer module on this chain and a matching module on the remote chain. We cannot select the port_id, this is whatever the local chain has bound the ibctransfer module to.
	Transfer *TransferMsg `json:"transfer,omitempty"`
	// Sends bank tokens owned by the contract to the given address on another chain. The channel must already be established between the ibctransfer module on this chain and a matching module on the remote chain. We cannot select the port_id, this is whatever the local chain has bound the ibctransfer module to.
	TransferV2 *TransferV2Msg `json:"transfer_v2,omitempty"`
	// Sends an IBC packet with given data over the existing channel. Data should be encoded in a format defined by the channel version, and the module on the other side should know how to parse this.
	SendPacket *SendPacketMsg `json:"send_packet,omitempty"`
	// Acknowledges a packet that this contract received over IBC. This allows acknowledging a packet that was not acknowledged yet in the `ibc_packet_receive` call.
	WriteAcknowledgement *WriteAcknowledgementMsg `json:"write_acknowledgement,omitempty"`
	// This will close an existing channel that is owned by this contract. Port is auto-assigned to the contract's IBC port
	CloseChannel *CloseChannelMsg `json:"close_channel,omitempty"`
	// Incentivizes the next IBC packet sent after this message with a fee. Note that this does not necessarily have to be a packet sent by this contract. The fees are taken from the contract's balance immediately and locked until the packet is handled.
	//
	// # Example
	//
	// Most commonly, you will attach this message to a response right before sending a packet using [`IbcMsg::SendPacket`] or [`IbcMsg::Transfer`].
	//
	// ```rust # use cosmwasm_std::{IbcMsg, IbcEndpoint, IbcFee, IbcTimeout, Coin, coins, CosmosMsg, Response, Timestamp};
	//
	// let incentivize = IbcMsg::PayPacketFee { port_id: "transfer".to_string(), channel_id: "source-channel".to_string(), fee: IbcFee { receive_fee: coins(100, "token"), ack_fee: coins(201, "token"), timeout_fee: coins(200, "token"), }, relayers: vec![], }; let transfer = IbcMsg::Transfer { channel_id: "source-channel".to_string(), to_address: "receiver".to_string(), amount: Coin::new(100u32, "token"), timeout: IbcTimeout::with_timestamp(Timestamp::from_nanos(0)), memo: None, };
	//
	// # #[cfg(feature = "stargate")] let _: Response = Response::new() .add_message(CosmosMsg::Ibc(incentivize)) .add_message(CosmosMsg::Ibc(transfer)); ```
	PayPacketFee *PayPacketFeeMsg `json:"pay_packet_fee,omitempty"`
	// Incentivizes the existing IBC packet with the given port, channel and sequence with a fee. Note that this does not necessarily have to be a packet sent by this contract. The fees are taken from the contract's balance immediately and locked until the packet is handled. They are added to the existing fees on the packet.
	PayPacketFeeAsync *PayPacketFeeAsyncMsg `json:"pay_packet_fee_async,omitempty"`
}
type Coin struct {
	Amount string `json:"amount"`
	Denom  string `json:"denom"`
}
type Hop struct {
	ChannelID string `json:"channel_id"`
	PortID    string `json:"port_id"`
}
type IBCAcknowledgement struct {
	Data []byte `json:"data"`
}
type IBCFee struct {
	AckFee     Array[Coin] `json:"ack_fee"`
	ReceiveFee Array[Coin] `json:"receive_fee"`
	TimeoutFee Array[Coin] `json:"timeout_fee"`
}

// In IBC each package must set at least one type of timeout: the timestamp or the block height. Using this rather complex enum instead of two timeout fields we ensure that at least one timeout is set.
type IBCTimeout struct {
	Block     *IBCTimeoutBlock `json:"block,omitempty"`
	Timestamp *Uint64          `json:"timestamp,omitempty"`
}

// IBCTimeoutHeight Height is a monotonically increasing data type that can be compared against another Height for the purposes of updating and freezing clients. Ordering is (revision_number, timeout_height)
type IBCTimeoutBlock struct {
	// block height after which the packet times out. the height within the given revision
	Height uint64 `json:"height"`
	// the version that the client is currently on (e.g. after resetting the chain this could increment 1 as height drops to 0)
	Revision uint64 `json:"revision"`
}