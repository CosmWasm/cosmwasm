#![allow(deprecated)]

use schemars::JsonSchema;
use std::fmt;

use crate::Binary;

use super::{attr, Attribute, CosmosMsg, Empty, Response};

#[deprecated(
    since = "0.14.0",
    note = "Use mutating helpers on Response/InitResponse/HandleResponse/MigrateResponse directly."
)]
#[derive(Clone, Debug, PartialEq)]
pub struct Context<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    messages: Vec<CosmosMsg<T>>,
    /// The attributes that will be emitted as part of a "wasm" event
    attributes: Vec<Attribute>,
    data: Option<Binary>,
}

impl<T> Default for Context<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    fn default() -> Self {
        Context {
            messages: vec![],
            attributes: vec![],
            data: None,
        }
    }
}

impl<T> Context<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_attribute<K: ToString, V: ToString>(&mut self, key: K, value: V) {
        self.attributes.push(attr(key, value));
    }

    pub fn add_message<U: Into<CosmosMsg<T>>>(&mut self, msg: U) {
        self.messages.push(msg.into());
    }

    pub fn set_data<U: Into<Binary>>(&mut self, data: U) {
        self.data = Some(data.into());
    }
}

impl<T> From<Context<T>> for Response<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    fn from(ctx: Context<T>) -> Self {
        Response {
            /// we do not support submessages here, as it was already deprecated when submessages were added
            submessages: vec![],
            messages: ctx.messages,
            attributes: ctx.attributes,
            data: ctx.data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{coins, BankMsg, HumanAddr, Response};

    #[test]
    fn empty_context() {
        let ctx = Context::new();

        let init: Response = ctx.clone().into();
        assert_eq!(init, Response::default());
    }

    #[test]
    fn full_context() {
        let mut ctx = Context::new();

        // build it up with the builder commands
        ctx.add_attribute("sender", &HumanAddr::from("john"));
        ctx.add_attribute("action", "test");
        ctx.add_message(BankMsg::Send {
            to_address: HumanAddr::from("foo"),
            amount: coins(128, "uint"),
        });
        ctx.set_data(b"banana");

        // and this is what is should return
        let expected_msgs = vec![CosmosMsg::Bank(BankMsg::Send {
            to_address: HumanAddr::from("foo"),
            amount: coins(128, "uint"),
        })];
        let expected_attributes = vec![attr("sender", "john"), attr("action", "test")];
        let expected_data = Some(Binary::from(b"banana"));

        let response: Response = ctx.clone().into();
        assert_eq!(&response.messages, &expected_msgs);
        assert_eq!(&response.attributes, &expected_attributes);
        assert_eq!(&response.data, &expected_data);
    }
}
