use crate::{IbcDstCallback, IbcMsg, IbcSrcCallback, IbcTimeout};

use super::{
    EmptyMemo, Forwarding, MemoSource, Token, WithCallbacks, WithDstCallback, WithMemo,
    WithSrcCallback,
};

impl<M: MemoSource> TransferMsgBuilderV2<M> {
    pub fn build(self) -> IbcMsg {
        IbcMsg::TransferV2 {
            channel_id: self.channel_id,
            to_address: self.to_address,
            tokens: self.tokens,
            timeout: self.timeout,
            memo: self.memo.into_memo(),
            forwarding: self.forwarding,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransferMsgBuilderV2<MemoData> {
    channel_id: String,
    to_address: String,
    tokens: Vec<Token>,
    timeout: IbcTimeout,
    memo: MemoData,
    forwarding: Forwarding,
}

impl TransferMsgBuilderV2<EmptyMemo> {
    /// Creates a new transfer message with the given parameters and no memo.
    pub fn new(
        channel_id: impl Into<String>,
        to_address: impl Into<String>,
        tokens: Vec<Token>,
        timeout: impl Into<IbcTimeout>,
    ) -> Self {
        Self {
            channel_id: channel_id.into(),
            to_address: to_address.into(),
            tokens,
            timeout: timeout.into(),
            memo: EmptyMemo,
            forwarding: Forwarding::default(),
        }
    }

    /// Adds a memo text to the transfer message.
    pub fn with_memo(self, memo: impl Into<String>) -> TransferMsgBuilderV2<WithMemo> {
        TransferMsgBuilderV2 {
            channel_id: self.channel_id,
            to_address: self.to_address,
            tokens: self.tokens,
            timeout: self.timeout,
            memo: WithMemo { memo: memo.into() },
            forwarding: self.forwarding,
        }
    }

    /// Adds an IBC source callback entry to the memo field.
    /// Use this if you want to receive IBC callbacks on the source chain.
    ///
    /// For more info check out [`crate::IbcSourceCallbackMsg`].
    pub fn with_src_callback(
        self,
        src_callback: IbcSrcCallback,
    ) -> TransferMsgBuilderV2<WithSrcCallback> {
        TransferMsgBuilderV2 {
            channel_id: self.channel_id,
            to_address: self.to_address,
            tokens: self.tokens,
            timeout: self.timeout,
            memo: WithSrcCallback { src_callback },
            forwarding: self.forwarding,
        }
    }

    /// Adds an IBC destination callback entry to the memo field.
    /// Use this if you want to receive IBC callbacks on the destination chain.
    ///
    /// For more info check out [`crate::IbcDestinationCallbackMsg`].
    pub fn with_dst_callback(
        self,
        dst_callback: IbcDstCallback,
    ) -> TransferMsgBuilderV2<WithDstCallback> {
        TransferMsgBuilderV2 {
            channel_id: self.channel_id,
            to_address: self.to_address,
            tokens: self.tokens,
            timeout: self.timeout,
            memo: WithDstCallback { dst_callback },
            forwarding: self.forwarding,
        }
    }
}

impl TransferMsgBuilderV2<WithSrcCallback> {
    /// Adds an IBC destination callback entry to the memo field.
    /// Use this if you want to receive IBC callbacks on the destination chain.
    ///
    /// For more info check out [`crate::IbcDestinationCallbackMsg`].
    pub fn with_dst_callback(
        self,
        dst_callback: IbcDstCallback,
    ) -> TransferMsgBuilderV2<WithCallbacks> {
        TransferMsgBuilderV2 {
            channel_id: self.channel_id,
            to_address: self.to_address,
            tokens: self.tokens,
            timeout: self.timeout,
            memo: WithCallbacks {
                src_callback: self.memo.src_callback,
                dst_callback,
            },
            forwarding: self.forwarding,
        }
    }
}

impl TransferMsgBuilderV2<WithDstCallback> {
    /// Adds an IBC source callback entry to the memo field.
    /// Use this if you want to receive IBC callbacks on the source chain.
    ///
    /// For more info check out [`crate::IbcSourceCallbackMsg`].
    pub fn with_src_callback(
        self,
        src_callback: IbcSrcCallback,
    ) -> TransferMsgBuilderV2<WithCallbacks> {
        TransferMsgBuilderV2 {
            channel_id: self.channel_id,
            to_address: self.to_address,
            tokens: self.tokens,
            timeout: self.timeout,
            memo: WithCallbacks {
                src_callback,
                dst_callback: self.memo.dst_callback,
            },
            forwarding: self.forwarding,
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

        let empty_memo_builder = TransferMsgBuilderV2::new(
            "channel-0",
            "cosmos1example",
            vec![coin(10, "ucoin").into()],
            Timestamp::from_seconds(12345),
        );

        let empty = empty_memo_builder.clone().build();
        let with_memo = empty_memo_builder.clone().with_memo("memo").build();

        let with_src_callback_builder = empty_memo_builder
            .clone()
            .with_src_callback(src_callback.clone());
        let with_src_callback = with_src_callback_builder.clone().build();
        let with_dst_callback_builder = empty_memo_builder
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
            empty,
            IbcMsg::TransferV2 {
                channel_id: "channel-0".to_string(),
                to_address: "cosmos1example".to_string(),
                tokens: vec![coin(10, "ucoin").into()],
                timeout: Timestamp::from_seconds(12345).into(),
                memo: None,
                forwarding: Forwarding::default()
            }
        );
        assert_eq!(
            with_memo,
            IbcMsg::TransferV2 {
                channel_id: "channel-0".to_string(),
                to_address: "cosmos1example".to_string(),
                tokens: vec![coin(10, "ucoin").into()],
                timeout: Timestamp::from_seconds(12345).into(),
                memo: Some("memo".to_string()),
                forwarding: Forwarding::default()
            }
        );
        assert_eq!(
            with_src_callback,
            IbcMsg::TransferV2 {
                channel_id: "channel-0".to_string(),
                to_address: "cosmos1example".to_string(),
                tokens: vec![coin(10, "ucoin").into()],
                timeout: Timestamp::from_seconds(12345).into(),
                memo: Some(
                    to_json_string(&IbcCallbackRequest::source(src_callback.clone())).unwrap()
                ),
                forwarding: Forwarding::default()
            }
        );
        assert_eq!(
            with_dst_callback,
            IbcMsg::TransferV2 {
                channel_id: "channel-0".to_string(),
                to_address: "cosmos1example".to_string(),
                tokens: vec![coin(10, "ucoin").into()],
                timeout: Timestamp::from_seconds(12345).into(),
                memo: Some(
                    to_json_string(&IbcCallbackRequest::destination(dst_callback.clone())).unwrap()
                ),
                forwarding: Forwarding::default()
            }
        );
        assert_eq!(
            with_both_callbacks1,
            IbcMsg::TransferV2 {
                channel_id: "channel-0".to_string(),
                to_address: "cosmos1example".to_string(),
                tokens: vec![coin(10, "ucoin").into()],
                timeout: Timestamp::from_seconds(12345).into(),
                memo: Some(
                    to_json_string(&IbcCallbackRequest::both(src_callback, dst_callback)).unwrap()
                ),
                forwarding: Forwarding::default(),
            }
        );
        assert_eq!(with_both_callbacks1, with_both_callbacks2);
    }
}
