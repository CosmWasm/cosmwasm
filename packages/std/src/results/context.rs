use schemars::JsonSchema;
use std::convert::TryFrom;
use std::fmt;

use crate::encoding::Binary;
use crate::errors::StdError;
use crate::types::Empty;

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
        Context::default()
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

impl<T> TryFrom<Context<T>> for InitResponse<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    type Error = StdError;

    fn try_from(ctx: Context<T>) -> Result<Self, Self::Error> {
        if ctx.data.is_some() {
            Err(StdError::generic_err(
                "cannot convert Context with data to InitResponse",
            ))
        } else {
            Ok(InitResponse {
                messages: ctx.messages,
                attributes: ctx.attributes,
            })
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
mod test {
    use super::super::{BankMsg, HandleResponse, InitResponse, MigrateResponse};
    use super::*;
    use crate::addresses::HumanAddr;
    use crate::coins;
    use crate::errors::{StdError, StdResult};
    use std::convert::TryInto;

    #[test]
    fn empty_context() {
        let ctx = Context::new();

        let init: InitResponse = ctx.clone().try_into().unwrap();
        assert_eq!(init, InitResponse::default());

        let init: HandleResponse = ctx.clone().try_into().unwrap();
        assert_eq!(init, HandleResponse::default());

        let init: MigrateResponse = ctx.clone().try_into().unwrap();
        assert_eq!(init, MigrateResponse::default());
    }

    #[test]
    fn full_context() {
        let mut ctx = Context::new();

        // build it up with the builder commands
        ctx.add_attribute("sender", &HumanAddr::from("john"));
        ctx.add_attribute("action", "test");
        ctx.add_message(BankMsg::Send {
            from_address: HumanAddr::from("goo"),
            to_address: HumanAddr::from("foo"),
            amount: coins(128, "uint"),
        });

        // and this is what is should return
        let expected_msgs = vec![CosmosMsg::Bank(BankMsg::Send {
            from_address: HumanAddr::from("goo"),
            to_address: HumanAddr::from("foo"),
            amount: coins(128, "uint"),
        })];
        let expected_attributes = vec![attr("sender", "john"), attr("action", "test")];
        let expected_data = Some(Binary::from(b"banana"));

        // try InitResponse before setting data
        let init: InitResponse = ctx.clone().try_into().unwrap();
        assert_eq!(&init.messages, &expected_msgs);
        assert_eq!(&init.attributes, &expected_attributes);

        ctx.set_data(b"banana");
        // should fail with data set
        let init_err: StdResult<InitResponse> = ctx.clone().try_into();
        match init_err.unwrap_err() {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, "cannot convert Context with data to InitResponse")
            }
            err => panic!("Unexpected error: {:?}", err),
        }

        // try Handle with everything set
        let handle: HandleResponse = ctx.clone().try_into().unwrap();
        assert_eq!(&handle.messages, &expected_msgs);
        assert_eq!(&handle.attributes, &expected_attributes);
        assert_eq!(&handle.data, &expected_data);

        // try Migrate with everything set
        let migrate: MigrateResponse = ctx.clone().try_into().unwrap();
        assert_eq!(&migrate.messages, &expected_msgs);
        assert_eq!(&migrate.attributes, &expected_attributes);
        assert_eq!(&migrate.data, &expected_data);
    }
}
