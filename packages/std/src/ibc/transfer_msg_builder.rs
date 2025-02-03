use crate::{
    to_json_string, Coin, IbcCallbackRequest, IbcDstCallback, IbcMsg, IbcSrcCallback, IbcTimeout,
};

// these are the different memo types and at the same time the states
// the TransferMsgBuilder can be in
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmptyMemo;
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct WithMemo {
    pub(crate) memo: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct WithSrcCallback {
    pub(crate) src_callback: IbcSrcCallback,
}
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct WithDstCallback {
    pub(crate) dst_callback: IbcDstCallback,
}
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct WithCallbacks {
    pub(crate) src_callback: IbcSrcCallback,
    pub(crate) dst_callback: IbcDstCallback,
}

pub trait MemoSource {
    fn into_memo(self) -> Option<String>;
}

impl MemoSource for EmptyMemo {
    fn into_memo(self) -> Option<String> {
        None
    }
}

impl MemoSource for WithMemo {
    fn into_memo(self) -> Option<String> {
        Some(self.memo)
    }
}

impl MemoSource for WithSrcCallback {
    fn into_memo(self) -> Option<String> {
        Some(to_json_string(&IbcCallbackRequest::source(self.src_callback)).unwrap())
    }
}

impl MemoSource for WithDstCallback {
    fn into_memo(self) -> Option<String> {
        Some(to_json_string(&IbcCallbackRequest::destination(self.dst_callback)).unwrap())
    }
}

impl MemoSource for WithCallbacks {
    fn into_memo(self) -> Option<String> {
        Some(
            to_json_string(&IbcCallbackRequest::both(
                self.src_callback,
                self.dst_callback,
            ))
            .unwrap(),
        )
    }
}

impl<M: MemoSource> TransferMsgBuilder<M> {
    pub fn build(self) -> IbcMsg {
        IbcMsg::Transfer {
            channel_id: self.channel_id,
            to_address: self.to_address,
            amount: self.amount,
            timeout: self.timeout,
            memo: self.memo.into_memo(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransferMsgBuilder<MemoData> {
    channel_id: String,
    to_address: String,
    amount: Coin,
    timeout: IbcTimeout,
    memo: MemoData,
}

impl TransferMsgBuilder<EmptyMemo> {
    /// Creates a new transfer message with the given parameters and no memo.
    pub fn new(
        channel_id: impl Into<String>,
        to_address: impl Into<String>,
        amount: Coin,
        timeout: impl Into<IbcTimeout>,
    ) -> Self {
        Self {
            channel_id: channel_id.into(),
            to_address: to_address.into(),
            amount,
            timeout: timeout.into(),
            memo: EmptyMemo,
        }
    }

    /// Adds a memo text to the transfer message.
    pub fn with_memo(self, memo: impl Into<String>) -> TransferMsgBuilder<WithMemo> {
        TransferMsgBuilder {
            channel_id: self.channel_id,
            to_address: self.to_address,
            amount: self.amount,
            timeout: self.timeout,
            memo: WithMemo { memo: memo.into() },
        }
    }

    /// Adds an IBC source callback entry to the memo field.
    /// Use this if you want to receive IBC callbacks on the source chain.
    ///
    /// For more info check out [`crate::IbcSourceCallbackMsg`].
    pub fn with_src_callback(
        self,
        src_callback: IbcSrcCallback,
    ) -> TransferMsgBuilder<WithSrcCallback> {
        TransferMsgBuilder {
            channel_id: self.channel_id,
            to_address: self.to_address,
            amount: self.amount,
            timeout: self.timeout,
            memo: WithSrcCallback { src_callback },
        }
    }

    /// Adds an IBC destination callback entry to the memo field.
    /// Use this if you want to receive IBC callbacks on the destination chain.
    ///
    /// For more info check out [`crate::IbcDestinationCallbackMsg`].
    pub fn with_dst_callback(
        self,
        dst_callback: IbcDstCallback,
    ) -> TransferMsgBuilder<WithDstCallback> {
        TransferMsgBuilder {
            channel_id: self.channel_id,
            to_address: self.to_address,
            amount: self.amount,
            timeout: self.timeout,
            memo: WithDstCallback { dst_callback },
        }
    }
}

impl TransferMsgBuilder<WithSrcCallback> {
    /// Adds an IBC destination callback entry to the memo field.
    /// Use this if you want to receive IBC callbacks on the destination chain.
    ///
    /// For more info check out [`crate::IbcDestinationCallbackMsg`].
    pub fn with_dst_callback(
        self,
        dst_callback: IbcDstCallback,
    ) -> TransferMsgBuilder<WithCallbacks> {
        TransferMsgBuilder {
            channel_id: self.channel_id,
            to_address: self.to_address,
            amount: self.amount,
            timeout: self.timeout,
            memo: WithCallbacks {
                src_callback: self.memo.src_callback,
                dst_callback,
            },
        }
    }
}

impl TransferMsgBuilder<WithDstCallback> {
    /// Adds an IBC source callback entry to the memo field.
    /// Use this if you want to receive IBC callbacks on the source chain.
    ///
    /// For more info check out [`crate::IbcSourceCallbackMsg`].
    pub fn with_src_callback(
        self,
        src_callback: IbcSrcCallback,
    ) -> TransferMsgBuilder<WithCallbacks> {
        TransferMsgBuilder {
            channel_id: self.channel_id,
            to_address: self.to_address,
            amount: self.amount,
            timeout: self.timeout,
            memo: WithCallbacks {
                src_callback,
                dst_callback: self.memo.dst_callback,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{coin, Addr, Timestamp, Uint64};

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

        let empty_memo_builder = TransferMsgBuilder::new(
            "channel-0",
            "cosmos1example",
            coin(10, "ucoin"),
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
            IbcMsg::Transfer {
                channel_id: "channel-0".to_string(),
                to_address: "cosmos1example".to_string(),
                amount: coin(10, "ucoin"),
                timeout: Timestamp::from_seconds(12345).into(),
                memo: None,
            }
        );
        assert_eq!(
            with_memo,
            IbcMsg::Transfer {
                channel_id: "channel-0".to_string(),
                to_address: "cosmos1example".to_string(),
                amount: coin(10, "ucoin"),
                timeout: Timestamp::from_seconds(12345).into(),
                memo: Some("memo".to_string()),
            }
        );
        assert_eq!(
            with_src_callback,
            IbcMsg::Transfer {
                channel_id: "channel-0".to_string(),
                to_address: "cosmos1example".to_string(),
                amount: coin(10, "ucoin"),
                timeout: Timestamp::from_seconds(12345).into(),
                memo: Some(
                    to_json_string(&IbcCallbackRequest::source(src_callback.clone())).unwrap()
                ),
            }
        );
        assert_eq!(
            with_dst_callback,
            IbcMsg::Transfer {
                channel_id: "channel-0".to_string(),
                to_address: "cosmos1example".to_string(),
                amount: coin(10, "ucoin"),
                timeout: Timestamp::from_seconds(12345).into(),
                memo: Some(
                    to_json_string(&IbcCallbackRequest::destination(dst_callback.clone())).unwrap()
                ),
            }
        );
        assert_eq!(
            with_both_callbacks1,
            IbcMsg::Transfer {
                channel_id: "channel-0".to_string(),
                to_address: "cosmos1example".to_string(),
                amount: coin(10, "ucoin"),
                timeout: Timestamp::from_seconds(12345).into(),
                memo: Some(
                    to_json_string(&IbcCallbackRequest::both(src_callback, dst_callback)).unwrap()
                ),
            }
        );
        assert_eq!(with_both_callbacks1, with_both_callbacks2);
    }
}
