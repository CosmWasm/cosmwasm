use crate::{Coin, IbcDstCallback, IbcMsg, IbcSrcCallback, Timestamp};

use super::{
    EmptyMemo, Hop, IbcTimeout, MemoSource, TransferV2Type, WithCallbacks, WithDstCallback,
    WithMemo, WithSrcCallback,
};

impl<M: MemoSource, F: Into<TransferV2Type>> TransferMsgBuilderV2<M, F> {
    pub fn build(self) -> IbcMsg {
        IbcMsg::TransferV2 {
            transfer_type: self.transfer_type.into(),
            to_address: self.to_address,
            tokens: self.tokens,
            memo: self.memo.into_memo(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransferMsgBuilderV2<MemoData, TransferType> {
    transfer_type: TransferType,
    to_address: String,
    tokens: Vec<Coin>,
    memo: MemoData,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct EmptyTransferType;

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Direct {
    transfer_type: TransferV2Type,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Forwarding {
    transfer_type: TransferV2Type,
}

impl From<Direct> for TransferV2Type {
    fn from(val: Direct) -> Self {
        val.transfer_type
    }
}

impl From<Forwarding> for TransferV2Type {
    fn from(val: Forwarding) -> Self {
        val.transfer_type
    }
}

pub trait ForwardingPossible {}
impl ForwardingPossible for WithMemo {}
impl ForwardingPossible for EmptyMemo {}
impl ForwardingPossible for WithDstCallback {}

pub trait SrcCallbackPossible {}
impl SrcCallbackPossible for Direct {}
impl SrcCallbackPossible for EmptyTransferType {}

pub trait AddDstCallbackPossible {
    type CallbackType;
    fn add_dst_callback(self, dst_callback: IbcDstCallback) -> Self::CallbackType;
}
impl AddDstCallbackPossible for WithSrcCallback {
    type CallbackType = WithCallbacks;
    fn add_dst_callback(self, dst_callback: IbcDstCallback) -> Self::CallbackType {
        WithCallbacks {
            src_callback: self.src_callback,
            dst_callback,
        }
    }
}
impl AddDstCallbackPossible for EmptyMemo {
    type CallbackType = WithDstCallback;
    fn add_dst_callback(self, dst_callback: IbcDstCallback) -> Self::CallbackType {
        WithDstCallback { dst_callback }
    }
}

pub trait AddSrcCallbackPossible {
    type CallbackType;
    fn add_src_callback(self, dst_callback: IbcSrcCallback) -> Self::CallbackType;
}
impl AddSrcCallbackPossible for WithDstCallback {
    type CallbackType = WithCallbacks;
    fn add_src_callback(self, src_callback: IbcSrcCallback) -> Self::CallbackType {
        WithCallbacks {
            dst_callback: self.dst_callback,
            src_callback,
        }
    }
}
impl AddSrcCallbackPossible for EmptyMemo {
    type CallbackType = WithSrcCallback;
    fn add_src_callback(self, src_callback: IbcSrcCallback) -> Self::CallbackType {
        WithSrcCallback { src_callback }
    }
}

impl TransferMsgBuilderV2<EmptyMemo, EmptyTransferType> {
    /// Creates a new transfer message with the given parameters and no memo.
    pub fn new(to_address: impl Into<String>, tokens: Vec<Coin>) -> Self {
        Self {
            transfer_type: EmptyTransferType {},
            to_address: to_address.into(),
            tokens,
            memo: EmptyMemo,
        }
    }
}

impl<TransferType> TransferMsgBuilderV2<EmptyMemo, TransferType> {
    /// Adds a memo text to the transfer message.
    pub fn with_memo(
        self,
        memo: impl Into<String>,
    ) -> TransferMsgBuilderV2<WithMemo, TransferType> {
        TransferMsgBuilderV2 {
            transfer_type: self.transfer_type,
            to_address: self.to_address,
            tokens: self.tokens,
            memo: WithMemo { memo: memo.into() },
        }
    }
}

impl<Memo: AddSrcCallbackPossible, TransferType: SrcCallbackPossible>
    TransferMsgBuilderV2<Memo, TransferType>
{
    /// Adds an IBC source callback entry to the memo field.
    /// Use this if you want to receive IBC callbacks on the source chain.
    ///
    /// For more info check out [`crate::IbcSourceCallbackMsg`].
    pub fn with_src_callback(
        self,
        src_callback: IbcSrcCallback,
    ) -> TransferMsgBuilderV2<Memo::CallbackType, TransferType> {
        TransferMsgBuilderV2 {
            transfer_type: self.transfer_type,
            to_address: self.to_address,
            tokens: self.tokens,
            memo: self.memo.add_src_callback(src_callback),
        }
    }
}

impl<Memo: AddDstCallbackPossible, TransferType> TransferMsgBuilderV2<Memo, TransferType> {
    /// Adds an IBC destination callback entry to the memo field.
    /// Use this if you want to receive IBC callbacks on the destination chain.
    ///
    /// For more info check out [`crate::IbcDestinationCallbackMsg`].
    pub fn with_dst_callback(
        self,
        dst_callback: IbcDstCallback,
    ) -> TransferMsgBuilderV2<Memo::CallbackType, TransferType> {
        TransferMsgBuilderV2 {
            transfer_type: self.transfer_type,
            to_address: self.to_address,
            tokens: self.tokens,
            memo: self.memo.add_dst_callback(dst_callback),
        }
    }
}

impl<Memo> TransferMsgBuilderV2<Memo, EmptyTransferType> {
    /// Creates a direct transfer without forwarding data.
    pub fn with_direct_transfer(
        self,
        channel_id: String,
        ibc_timeout: IbcTimeout,
    ) -> TransferMsgBuilderV2<Memo, Direct> {
        TransferMsgBuilderV2 {
            transfer_type: Direct {
                transfer_type: TransferV2Type::Direct {
                    channel_id,
                    ibc_timeout,
                },
            },
            to_address: self.to_address,
            tokens: self.tokens,
            memo: self.memo,
        }
    }
}

impl<Memo: ForwardingPossible> TransferMsgBuilderV2<Memo, EmptyTransferType> {
    /// Adds forwarding data.
    /// More information can be found in the IBC doc:
    /// https://ibc.cosmos.network/main/apps/transfer/messages/#msgtransfer
    ///
    /// It is worth to notice that the builder does not allow to add forwarding data along with
    /// source callback. It is discouraged in the IBC docs:
    /// https://ibc.cosmos.network/v9/middleware/callbacks/overview/#known-limitations
    pub fn with_forwarding(
        self,
        channel_id: String,
        hops: Vec<Hop>,
        timeout: Timestamp,
    ) -> TransferMsgBuilderV2<Memo, Forwarding> {
        TransferMsgBuilderV2 {
            transfer_type: Forwarding {
                transfer_type: TransferV2Type::MultiHop {
                    channel_id,
                    hops,
                    timeout,
                },
            },
            to_address: self.to_address,
            tokens: self.tokens,
            memo: self.memo,
        }
    }

    /// Adds forwarding data with an unwinding flag set.
    /// More information can be found in the IBC doc:
    /// https://ibc.cosmos.network/main/apps/transfer/messages/#msgtransfer
    ///
    /// It is worth to notice that the builder does not allow to add forwarding data along with
    /// source callback. It is discouraged in the IBC docs:
    /// https://ibc.cosmos.network/v9/middleware/callbacks/overview/#known-limitations
    pub fn with_forwarding_unwinding(
        self,
        hops: Vec<Hop>,
        timeout: Timestamp,
    ) -> TransferMsgBuilderV2<Memo, Forwarding> {
        TransferMsgBuilderV2 {
            transfer_type: Forwarding {
                transfer_type: TransferV2Type::Unwinding { hops, timeout },
            },
            to_address: self.to_address,
            tokens: self.tokens,
            memo: self.memo,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{coin, to_json_string, Addr, IbcCallbackRequest, Timestamp, Uint64};

    use super::*;

    #[test]
    fn test_transfer_msg_builder() {
        let src_callback = IbcSrcCallback {
            address: Addr::unchecked("src"),
            gas_limit: Some(Uint64::new(12345)),
        };
        let dst_callback = IbcDstCallback {
            address: "dst".to_string(),
            gas_limit: None,
        };

        let empty_builder = TransferMsgBuilderV2::new("cosmos1example", vec![coin(10, "ucoin")]);

        let direct_builder = empty_builder.clone().with_direct_transfer(
            "channel-0".to_owned(),
            IbcTimeout::with_timestamp(Timestamp::from_seconds(12345)),
        );
        let direct = direct_builder.clone().build();

        let forwarding_builder = empty_builder.clone().with_forwarding(
            "channel-0".to_owned(),
            vec![Hop {
                port_id: "port-id".to_owned(),
                channel_id: "channel-id".to_owned(),
            }],
            Timestamp::from_seconds(12345),
        );
        let forwarding = forwarding_builder.clone().build();

        let unwinding_builder = empty_builder.clone().with_forwarding_unwinding(
            vec![Hop {
                port_id: "port-id".to_owned(),
                channel_id: "channel-id".to_owned(),
            }],
            Timestamp::from_seconds(12345),
        );
        let unwinding = unwinding_builder
            .with_dst_callback(dst_callback.clone())
            .build();

        let with_memo = forwarding_builder.with_memo("memo").build();

        let with_src_callback_builder = direct_builder
            .clone()
            .with_src_callback(src_callback.clone());
        let with_src_callback = with_src_callback_builder.clone().build();
        let with_dst_callback_builder = direct_builder
            .clone()
            .with_dst_callback(dst_callback.clone());
        let with_dst_callback = with_dst_callback_builder.clone().build();

        let with_both_callbacks1 = with_src_callback_builder
            .with_dst_callback(dst_callback.clone())
            .build();

        let with_both_callbacks2 = with_dst_callback_builder
            .with_src_callback(src_callback.clone())
            .build();

        // assert all the different messages
        assert_eq!(
            direct,
            IbcMsg::TransferV2 {
                transfer_type: TransferV2Type::Direct {
                    channel_id: "channel-0".to_string(),
                    ibc_timeout: Timestamp::from_seconds(12345).into()
                },
                to_address: "cosmos1example".to_string(),
                tokens: vec![coin(10, "ucoin")],
                memo: None,
            }
        );
        assert_eq!(
            forwarding,
            IbcMsg::TransferV2 {
                transfer_type: TransferV2Type::MultiHop {
                    channel_id: "channel-0".to_string(),
                    hops: vec![Hop {
                        port_id: "port-id".to_owned(),
                        channel_id: "channel-id".to_owned()
                    }],
                    timeout: Timestamp::from_seconds(12345)
                },
                to_address: "cosmos1example".to_string(),
                tokens: vec![coin(10, "ucoin")],
                memo: None,
            }
        );
        assert_eq!(
            with_memo,
            IbcMsg::TransferV2 {
                transfer_type: TransferV2Type::MultiHop {
                    channel_id: "channel-0".to_string(),
                    hops: vec![Hop {
                        port_id: "port-id".to_owned(),
                        channel_id: "channel-id".to_owned()
                    }],
                    timeout: Timestamp::from_seconds(12345)
                },
                to_address: "cosmos1example".to_string(),
                tokens: vec![coin(10, "ucoin")],
                memo: Some("memo".to_string()),
            }
        );
        assert_eq!(
            with_src_callback,
            IbcMsg::TransferV2 {
                transfer_type: TransferV2Type::Direct {
                    channel_id: "channel-0".to_string(),
                    ibc_timeout: Timestamp::from_seconds(12345).into()
                },
                to_address: "cosmos1example".to_string(),
                tokens: vec![coin(10, "ucoin")],
                memo: Some(
                    to_json_string(&IbcCallbackRequest::source(src_callback.clone())).unwrap()
                ),
            }
        );
        assert_eq!(
            with_dst_callback,
            IbcMsg::TransferV2 {
                transfer_type: TransferV2Type::Direct {
                    channel_id: "channel-0".to_string(),
                    ibc_timeout: Timestamp::from_seconds(12345).into()
                },
                to_address: "cosmos1example".to_string(),
                tokens: vec![coin(10, "ucoin")],
                memo: Some(
                    to_json_string(&IbcCallbackRequest::destination(dst_callback.clone())).unwrap()
                ),
            }
        );
        assert_eq!(
            with_both_callbacks1,
            IbcMsg::TransferV2 {
                transfer_type: TransferV2Type::Direct {
                    channel_id: "channel-0".to_string(),
                    ibc_timeout: Timestamp::from_seconds(12345).into()
                },
                to_address: "cosmos1example".to_string(),
                tokens: vec![coin(10, "ucoin")],
                memo: Some(
                    to_json_string(&IbcCallbackRequest::both(
                        src_callback,
                        dst_callback.clone()
                    ))
                    .unwrap()
                ),
            }
        );
        assert_eq!(with_both_callbacks1, with_both_callbacks2);
        assert_eq!(
            unwinding,
            IbcMsg::TransferV2 {
                transfer_type: TransferV2Type::Unwinding {
                    hops: vec![Hop {
                        port_id: "port-id".to_owned(),
                        channel_id: "channel-id".to_owned()
                    }],
                    timeout: Timestamp::from_seconds(12345)
                },
                to_address: "cosmos1example".to_string(),
                tokens: vec![coin(10, "ucoin")],
                memo: Some(to_json_string(&IbcCallbackRequest::destination(dst_callback)).unwrap()),
            }
        );
    }
}
