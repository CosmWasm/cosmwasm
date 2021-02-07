use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{Binary, Empty};

use super::attribute::Attribute;
use super::cosmos_msg::CosmosMsg;
use super::mut_response::MutResponse;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitResponse<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    pub messages: Vec<CosmosMsg<T>>,
    /// The attributes that will be emitted as part of a "wasm" event
    pub attributes: Vec<Attribute>,
    pub data: Option<Binary>,
}

impl<T> InitResponse<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T> Default for InitResponse<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    fn default() -> Self {
        InitResponse {
            messages: vec![],
            attributes: vec![],
            data: None,
        }
    }
}

impl<T> MutResponse<T> for InitResponse<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    fn add_attribute<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
        self.attributes.push(Attribute {
            key: key.into(),
            value: value.into(),
        });
    }

    fn add_message<U: Into<CosmosMsg<T>>>(&mut self, msg: U) {
        self.messages.push(msg.into());
    }

    fn set_data<U: Into<Binary>>(&mut self, data: U) {
        self.data = Some(data.into());
    }
}

#[cfg(test)]
mod tests {
    use super::super::BankMsg;
    use super::*;
    use crate::addresses::HumanAddr;
    use crate::{coins, from_slice, to_vec};

    #[test]
    fn can_serialize_and_deserialize_init_response() {
        let original = InitResponse {
            messages: vec![BankMsg::Send {
                to_address: HumanAddr::from("you"),
                amount: coins(1015, "earth"),
            }
            .into()],
            attributes: vec![Attribute {
                key: "action".to_string(),
                value: "release".to_string(),
            }],
            data: Some(Binary::from([0xAA, 0xBB])),
        };
        let serialized = to_vec(&original).expect("encode contract result");
        let deserialized: InitResponse = from_slice(&serialized).expect("decode contract result");
        assert_eq!(deserialized, original);
    }
}
