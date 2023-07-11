pub trait SerializeForBasicType {
    fn serialize(&self, buf: &mut Vec<u8>);
    fn deserialize(buf: &[u8]) -> Option<Self>
    where
        Self: Sized;
}

impl SerializeForBasicType for u32 {
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
    fn deserialize(buf: &[u8]) -> Option<Self> {
        if buf.len() < 4 {
            return None;
        }
        Some(u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]))
    }
}

impl SerializeForBasicType for u64 {
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
    fn deserialize(buf: &[u8]) -> Option<Self> {
        if buf.len() < 8 {
            return None;
        }
        Some(u64::from_be_bytes([
            buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
        ]))
    }
}

impl SerializeForBasicType for String {
    fn serialize(&self, buf: &mut Vec<u8>) {
        let bytes = self.as_bytes();
        buf.extend_from_slice(bytes);
    }
    fn deserialize(buf: &[u8]) -> Option<Self> {
        Some(String::from_utf8_lossy(buf).to_string())
    }
}

#[allow(dead_code)]
pub fn serialize_to_bytes<T: SerializeForBasicType>(value: &T) -> Vec<u8> {
    let mut buf = Vec::new();
    value.serialize(&mut buf);
    buf
}

pub fn deserialize_from_bytes<T: SerializeForBasicType>(bytes: Vec<u8>) -> Option<T> {
    T::deserialize(&bytes)
}

#[cfg(test)]
mod tests {
    use crate::serde_basic_type::{deserialize_from_bytes, serialize_to_bytes};

    #[test]
    fn test_u32() {
        let array: [u32; 4] = [std::u32::MIN, 1000, 99999, std::u32::MAX];
        for element in array.iter() {
            let ser_data = serialize_to_bytes(element);
            let value: u32 = deserialize_from_bytes(ser_data).unwrap();
            assert_eq!(*element, value)
        }
    }

    #[test]
    fn test_u64() {
        let array: [u64; 4] = [std::u64::MIN, 1000, 99999, std::u64::MAX];
        for element in array.iter() {
            let ser_data = serialize_to_bytes(element);
            let value: u64 = deserialize_from_bytes(ser_data).unwrap();
            assert_eq!(*element, value)
        }
    }

    #[test]
    fn test_string() {
        let array: [String; 4] = [
            "okt to the moon".to_string(),
            "abc".to_string(),
            "hello world".to_string(),
            "hello_world".to_string(),
        ];
        for element in array.iter() {
            let ser_data = serialize_to_bytes(element);
            let value: String = deserialize_from_bytes(ser_data).unwrap();
            assert_eq!(*element, value)
        }
    }
}
