use prost::{DecodeError, EncodeError, Message};

pub fn from_slice<T: Message + Default>(data: &[u8]) -> Result<T, DecodeError> {
    T::decode(data)
}

pub fn to_vec<T: Message + Default>(obj: &T) -> Result<Vec<u8>, EncodeError> {
    let mut buf = Vec::<u8>::new();
    obj.encode(&mut buf)?;
    Ok(buf)
}