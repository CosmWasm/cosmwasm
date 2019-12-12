// these two are conditionally compiled, only for wasm32
pub mod exports;
pub mod imports;

pub mod errors;
pub mod memory;
pub mod mock;
pub mod query;
pub mod serde;
pub mod storage;
pub mod types;


/** demo **/

use snafu::ResultExt;

pub trait Request: ::serde::ser::Serialize {
    type Response: ::serde::de::DeserializeOwned;

    fn ser(&self) -> crate::errors::Result<Vec<u8>> {
        serde::to_vec(&self).context(crate::errors::SerializeErr{kind: "Request"})
    }

    fn de(output: &[u8]) -> crate::errors::Result<Self::Response> {
        serde::from_slice(output).context(crate::errors::ParseErr{kind: "Request"})
    }
}

pub fn call<T: Request>(data: T) -> crate::errors::Result<T::Response> {
    let output = do_call(&data.ser()?)?;
    T::de(&output)
}

fn do_call(data: &[u8]) -> crate::errors::Result<Vec<u8>> {
    // TODO: this makes some external function call
    Ok(b"{}".to_vec())
}

/** try using this... see what the compiler says **/

// Note: this doesn't work as written.
// Maybe we would need to try an approach like https://docs.rs/refl/0.2.0/refl/

pub enum MyRequest {
    A(i32, i32),
    B(String),
}

pub enum MyResponse {
    A(i32),
    B(Vec<u8>),
}

// This won't compile, cannot impl on enum varinats
// Relevant (open) RFC is at https://github.com/rust-lang/rfcs/pull/2593
impl Request for MyRequest::A {
    type Response = MyResponse::A;
}

impl Request for MyRequest::B {
    type Response = MyResponse::B;
}

fn demo() {
    let a = MyRequest::A(7, 12);
    let b = call(a).unwrap();
    assert_eq!(b.0, 19);
}