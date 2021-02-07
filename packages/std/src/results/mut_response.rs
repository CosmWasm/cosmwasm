use schemars::JsonSchema;
use std::fmt;

use crate::{Binary, CosmosMsg};

/// A trait for mutating helpers on response types
/// ([`InitResponse`], [`MigrateResponse`] and [`HandleResponse`]).
/// Use this to create a mutable response instance early in your contract's
/// logic and incrementally add to it.
///
/// # Examples
///
/// ```
/// # use cosmwasm_std::{coins, BankMsg, Binary, DepsMut, Env, HumanAddr, MessageInfo, MigrateResponse};
/// # type InitMsg = ();
/// # type MyError = ();
/// use cosmwasm_std::{InitResponse, MutResponse};
///
/// pub fn init(
///     deps: DepsMut,
///     _env: Env,
///     info: MessageInfo,
///     msg: InitMsg,
/// ) -> Result<InitResponse, MyError> {
///     let mut response = InitResponse::new();
///     // ...
///     response.add_attribute("Let the", "hacking begin");
///     // ...
///     response.add_message(BankMsg::Send {
///         to_address: HumanAddr::from("recipient"),
///         amount: coins(128, "uint"),
///     });
///     response.add_attribute("foo", "bar");
///     // ...
///     response.set_data(Binary::from(b"the result data"));
///     Ok(response)
/// }
/// ```
pub trait MutResponse<T>: Default
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    /// Creates an empty response.
    /// Only use `new` when creating a mutable response object.
    /// In all other cases use `Default::default()` or construct the response directly.
    fn new() -> Self {
        Default::default()
    }

    fn add_attribute<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V);

    fn add_message<U: Into<CosmosMsg<T>>>(&mut self, msg: U);

    fn set_data<U: Into<Binary>>(&mut self, data: U);
}
