use schemars::JsonSchema;
use std::fmt;

use crate::{Binary, Empty};

use super::attribute::{attr, Attribute};
use super::cosmos_msg::CosmosMsg;
use super::handle::HandleResponse;
use super::init::InitResponse;
use super::migrate::MigrateResponse;

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

impl<T> From<Context<T>> for InitResponse<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    fn from(ctx: Context<T>) -> Self {
        InitResponse {
            messages: ctx.messages,
            attributes: ctx.attributes,
            data: ctx.data,
        }
    }
}

impl<T> From<Context<T>> for HandleResponse<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    fn from(ctx: Context<T>) -> Self {
        HandleResponse {
            messages: ctx.messages,
            attributes: ctx.attributes,
            data: ctx.data,
        }
    }
}

impl<T> From<Context<T>> for MigrateResponse<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    fn from(ctx: Context<T>) -> Self {
        MigrateResponse {
            messages: ctx.messages,
            attributes: ctx.attributes,
            data: ctx.data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{BankMsg, HandleResponse, InitResponse, MigrateResponse};
    use super::*;
    use crate::addresses::HumanAddr;
    use crate::coins;

    #[test]
    fn empty_context() {
        let ctx = Context::new();

        let init: InitResponse = ctx.clone().into();
        assert_eq!(init, InitResponse::default());

        let init: HandleResponse = ctx.clone().into();
        assert_eq!(init, HandleResponse::default());

        let init: MigrateResponse = ctx.clone().into();
        assert_eq!(init, MigrateResponse::default());
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

        // try InitResponse
        let init: InitResponse = ctx.clone().into();
        assert_eq!(&init.messages, &expected_msgs);
        assert_eq!(&init.attributes, &expected_attributes);
        assert_eq!(&init.data, &expected_data);

        // try Handle with everything set
        let handle: HandleResponse = ctx.clone().into();
        assert_eq!(&handle.messages, &expected_msgs);
        assert_eq!(&handle.attributes, &expected_attributes);
        assert_eq!(&handle.data, &expected_data);

        // try Migrate with everything set
        let migrate: MigrateResponse = ctx.clone().into();
        assert_eq!(&migrate.messages, &expected_msgs);
        assert_eq!(&migrate.attributes, &expected_attributes);
        assert_eq!(&migrate.data, &expected_data);
    }
}
