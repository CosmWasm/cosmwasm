use serde::Deserialize;

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub num_tests: usize,
    pub tests: Vec<Test>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Test {
    pub tc_id: i64,
    #[serde(deserialize_with = "hex::deserialize")]
    pub public_key_uncompressed: Vec<u8>,
    #[serde(deserialize_with = "hex::deserialize")]
    pub msg: Vec<u8>,
    pub sig: Sig,
    pub comment: String,
    pub valid: bool,
    pub flags: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sig {
    #[serde(deserialize_with = "hex::deserialize")]
    pub r: Vec<u8>,
    #[serde(deserialize_with = "hex::deserialize")]
    pub s: Vec<u8>,
    pub id: u8,
}

pub fn read_file(path: &str) -> File {
    use std::fs::File;
    use std::io::BufReader;

    // Open the file in read-only mode with buffer.
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    serde_json::from_reader(reader).unwrap()
}

pub fn combine_signature(sig: &Sig) -> Vec<u8> {
    // the test data contains values with leading zeroes, which we need to ignore
    let first_non_zero = sig.r.iter().position(|&v| v != 0).unwrap_or_default();
    let r = &sig.r[first_non_zero..];
    let first_non_zero = sig.s.iter().position(|&v| v != 0).unwrap_or_default();
    let s = &sig.s[first_non_zero..];

    // at least one of the tests has an s that is 33 bytes long
    let r_len = r.len().max(32);
    let s_len = s.len().max(32);

    // the test data also contains values with less than 32 bytes, so we need to pad them with zeroes
    let mut signature = vec![0; r_len + s_len];
    let (r_part, s_part) = signature.split_at_mut(r_len);
    r_part[r_len - r.len()..].copy_from_slice(r);
    s_part[s_len - s.len()..].copy_from_slice(s);

    signature
}
