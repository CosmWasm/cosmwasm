use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{to_binary, to_vec, Binary, StdResult};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum JsonAck<T> {
    Result(T),
    Error(String),
}

impl<T: Serialize> JsonAck<T> {
    /// Creates a success ack by serializing the data with JSON.
    pub fn success(data: T) -> Self {
        JsonAck::Result(data)
    }

    /// Creates an error ack
    pub fn error(err: impl Into<String>) -> Self {
        JsonAck::Error(err.into())
    }

    #[must_use = "if you intended to assert that this is a success, consider `.unwrap()` instead"]
    #[inline]
    pub const fn is_success(&self) -> bool {
        matches!(*self, JsonAck::Result(_))
    }

    #[must_use = "if you intended to assert that this is an error, consider `.unwrap_err()` instead"]
    #[inline]
    pub const fn is_error(&self) -> bool {
        !self.is_success()
    }

    /// Serialized the ack to binary using JSON. This used for setting the acknowledgement
    /// field in IbcReceiveResponse.
    ///
    /// ## Examples
    ///
    /// Show how the acknowledgement looks on the write:
    ///
    /// ```
    /// # use cosmwasm_std::{Binary, JsonAck};
    /// // 0x01 is a FungibleTokenPacketSuccess from ICS-20.
    /// let ack1: JsonAck<Binary> = JsonAck::success(Binary::from([0x01]));
    /// assert_eq!(ack1.to_binary().unwrap(), br#"{"result":"AQ=="}"#);
    ///
    /// let ack2: JsonAck<Binary> = JsonAck::error("kaputt"); // Some free text error message
    /// assert_eq!(ack2.to_binary().unwrap(), br#"{"error":"kaputt"}"#);
    /// ```
    ///
    /// Set acknowledgement field in `IbcReceiveResponse`:
    ///
    /// ```ignore
    /// use cosmwasm_std::{Binary, JsonAck, IbcReceiveResponse};
    ///
    /// // 0x01 is a FungibleTokenPacketSuccess from ICS-20.
    /// let ack: JsonAck<Binary> = JsonAck::success(Binary::from([0x01]));
    ///
    /// let res = IbcReceiveResponse::new().set_ack(ack.to_binary());
    /// let res = IbcReceiveResponse::new().set_ack(ack); // Does the same but consumes the instance
    /// ```
    pub fn to_binary(&self) -> StdResult<Binary> {
        to_binary(&self)
    }

    pub fn to_string(&self) -> StdResult<String> {
        let json_bin = to_vec(&self)?;
        let json_string = unsafe { String::from_utf8_unchecked(json_bin) };
        Ok(json_string)
    }

    pub fn unwrap(self) -> T {
        match self {
            JsonAck::Result(data) => data,
            JsonAck::Error(err) => panic!("{}", err),
        }
    }

    pub fn unwrap_err(self) -> String {
        match self {
            JsonAck::Result(_) => panic!("not an error"),
            JsonAck::Error(err) => err,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Empty, Uint128, Uint64};

    use super::*;

    #[test]
    fn jsonack_success_works() {
        let success = JsonAck::success(b"foo");
        match success {
            JsonAck::Result(data) => assert_eq!(data, b"foo"),
            JsonAck::Error(_err) => panic!("must not be an error"),
        }
    }

    #[test]
    fn jsonack_error_works() {
        let err = JsonAck::<Empty>::error("bar");
        match err {
            JsonAck::Result(_data) => panic!("must not be a success"),
            JsonAck::Error(err) => assert_eq!(err, "bar"),
        }
    }

    #[test]
    fn jsonack_is_success_is_error_work() {
        let success = JsonAck::<&str>::success("foo");
        let err = JsonAck::<&str>::error("bar");
        // is_success
        assert!(success.is_success());
        assert!(!err.is_success());
        // is_eror
        assert!(!success.is_error());
        assert!(err.is_error());
    }

    #[test]
    fn jsonack_to_string_works() {
        // Binary data in array and vector becomes an array in JSON. Not great but consistent.
        let ack = JsonAck::success(b"\x01");
        assert_eq!(ack.to_string().unwrap(), r#"{"result":[1]}"#);
        let ack = JsonAck::success(vec![0x01]);
        assert_eq!(ack.to_string().unwrap(), r#"{"result":[1]}"#);
        // But when we use Binary instead, we get a base64 string
        let ack = JsonAck::success(Binary::from([0x01]));
        assert_eq!(ack.to_string().unwrap(), r#"{"result":"AQ=="}"#);

        // Numeric acks
        let ack = JsonAck::<u32>::success(75);
        assert_eq!(ack.to_string().unwrap(), r#"{"result":75}"#);
        let ack = JsonAck::<i64>::success(-46518);
        assert_eq!(ack.to_string().unwrap(), r#"{"result":-46518}"#);
        let ack = JsonAck::<Uint64>::success(Uint64::new(32));
        assert_eq!(ack.to_string().unwrap(), r#"{"result":"32"}"#);
        let ack = JsonAck::<Uint128>::success(Uint128::new(684684787));
        assert_eq!(ack.to_string().unwrap(), r#"{"result":"684684787"}"#);

        // Strucures serialize as nested JSON
        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
        #[serde(rename_all = "snake_case")]
        struct Foo {
            count: u32,
        }
        let ack = JsonAck::success(Foo { count: 78 });
        assert_eq!(ack.to_string().unwrap(), r#"{"result":{"count":78}}"#);

        // Error
        let ack = JsonAck::<Empty>::error("kaputt");
        assert_eq!(ack.to_string().unwrap(), r#"{"error":"kaputt"}"#);
    }
}
