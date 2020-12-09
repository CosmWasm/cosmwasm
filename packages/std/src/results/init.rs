use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::errors::StdError;
use crate::types::Empty;

use super::attribute::Attribute;
use super::cosmos_msg::CosmosMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitResponse<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    pub messages: Vec<CosmosMsg<T>>,
    /// The attributes that will be emitted as part of a "wasm" event
    pub attributes: Vec<Attribute>,
}

impl<T> Default for InitResponse<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    fn default() -> Self {
        InitResponse {
            messages: vec![],
            attributes: vec![],
        }
    }
}

#[deprecated(
    since = "0.12.1",
    note = "InitResult is deprecated because it uses StdError, which should be replaced with custom errors in CosmWasm 0.11+. \
            Replace this with Result<InitResponse, StdError> or Result<InitResponse<U>, StdError> and consider migrating to custom errors from there."
)]
pub type InitResult<U = Empty> = Result<InitResponse<U>, StdError>;

#[cfg(test)]
mod test {
    use super::super::BankMsg;
    use super::*;
    use crate::addresses::HumanAddr;
    use crate::{coins, from_slice, to_vec};

    #[test]
    fn can_serialize_and_deserialize_init_response() {
        let original = InitResponse {
            messages: vec![BankMsg::Send {
                from_address: HumanAddr::from("me"),
                to_address: HumanAddr::from("you"),
                amount: coins(1015, "earth"),
            }
            .into()],
            attributes: vec![Attribute {
                key: "action".to_string(),
                value: "release".to_string(),
            }],
        };
        let serialized = to_vec(&original).expect("encode contract result");
        let deserialized: InitResponse = from_slice(&serialized).expect("decode contract result");
        assert_eq!(deserialized, original);
    }
}
