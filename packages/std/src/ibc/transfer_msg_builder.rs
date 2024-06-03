use std::marker::PhantomData;

use crate::{
    to_json_string, Coin, IbcCallbackRequest, IbcDstCallback, IbcMsg, IbcSrcCallback, IbcTimeout,
};

// these are the different states the TransferMsgBuilder can be in
#[derive(Clone, Debug, PartialEq, Eq)]
struct EmptyMemo;
#[derive(Clone, Debug, PartialEq, Eq)]
struct WithMemo;
#[derive(Clone, Debug, PartialEq, Eq)]
struct WithSrcCallback;
#[derive(Clone, Debug, PartialEq, Eq)]
struct WithDstCallback;
#[derive(Clone, Debug, PartialEq, Eq)]
struct WithCallbacks;

// TODO: use trait for MemoData and get rid of state?
#[derive(Clone, Debug, PartialEq, Eq)]
enum MemoData {
    Empty,
    Text(String),
    IbcCallbacks(IbcCallbackRequest),
}

impl From<MemoData> for Option<String> {
    fn from(memo: MemoData) -> Option<String> {
        match memo {
            MemoData::Empty => None,
            MemoData::Text(text) => Some(text),
            MemoData::IbcCallbacks(callbacks) => Some(to_json_string(&callbacks).unwrap()),
        }
    }
}

impl<T> TransferMsgBuilder<T> {
    pub fn build(self) -> IbcMsg {
        IbcMsg::Transfer {
            channel_id: self.channel_id,
            to_address: self.to_address,
            amount: self.amount,
            timeout: self.timeout,
            memo: self.memo.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransferMsgBuilder<State> {
    channel_id: String,
    to_address: String,
    amount: Coin,
    timeout: IbcTimeout,
    memo: MemoData,
    _state: PhantomData<State>,
}

impl TransferMsgBuilder<EmptyMemo> {
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
            memo: MemoData::Empty,
            _state: PhantomData,
        }
    }

    pub fn with_memo(self, memo: impl Into<String>) -> TransferMsgBuilder<WithMemo> {
        TransferMsgBuilder {
            channel_id: self.channel_id,
            to_address: self.to_address,
            amount: self.amount,
            timeout: self.timeout,
            memo: MemoData::Text(memo.into()),
            _state: PhantomData,
        }
    }

    pub fn with_src_callback(
        self,
        src_callback: IbcSrcCallback,
    ) -> TransferMsgBuilder<WithSrcCallback> {
        TransferMsgBuilder {
            channel_id: self.channel_id,
            to_address: self.to_address,
            amount: self.amount,
            timeout: self.timeout,
            memo: MemoData::IbcCallbacks(IbcCallbackRequest::source(src_callback)),
            _state: PhantomData,
        }
    }

    pub fn with_dst_callback(
        self,
        dst_callback: IbcDstCallback,
    ) -> TransferMsgBuilder<WithDstCallback> {
        TransferMsgBuilder {
            channel_id: self.channel_id,
            to_address: self.to_address,
            amount: self.amount,
            timeout: self.timeout,
            memo: MemoData::IbcCallbacks(IbcCallbackRequest::destination(dst_callback)),
            _state: PhantomData,
        }
    }
}

impl TransferMsgBuilder<WithSrcCallback> {
    pub fn with_dst_callback(
        self,
        dst_callback: IbcDstCallback,
    ) -> TransferMsgBuilder<WithCallbacks> {
        TransferMsgBuilder {
            channel_id: self.channel_id,
            to_address: self.to_address,
            amount: self.amount,
            timeout: self.timeout,
            memo: match self.memo {
                MemoData::IbcCallbacks(IbcCallbackRequest {
                    src_callback: Some(src_callback),
                    ..
                }) => MemoData::IbcCallbacks(IbcCallbackRequest::both(src_callback, dst_callback)),
                _ => unreachable!(), // we know this never happens because of the WithSrcCallback state
            },
            _state: PhantomData,
        }
    }
}

impl TransferMsgBuilder<WithDstCallback> {
    pub fn with_src_callback(
        self,
        src_callback: IbcSrcCallback,
    ) -> TransferMsgBuilder<WithCallbacks> {
        TransferMsgBuilder {
            channel_id: self.channel_id,
            to_address: self.to_address,
            amount: self.amount,
            timeout: self.timeout,
            memo: match self.memo {
                MemoData::IbcCallbacks(IbcCallbackRequest {
                    dest_callback: Some(dst_callback),
                    ..
                }) => MemoData::IbcCallbacks(IbcCallbackRequest::both(src_callback, dst_callback)),
                _ => unreachable!(), // we know this never happens because of the WithDstCallback state
            },
            _state: PhantomData,
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
