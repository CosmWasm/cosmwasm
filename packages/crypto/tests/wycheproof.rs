use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub number_of_tests: usize,
    pub test_groups: Vec<TestGroup>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TestGroup {
    pub public_key: Key,
    pub tests: Vec<TestCase>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Key {
    pub uncompressed: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TestCase {
    pub tc_id: u32,
    pub comment: String,
    pub msg: String,
    pub sig: String,
    // "acceptable", "valid" or "invalid"
    pub result: String,
}

pub fn read_file(path: &str) -> File {
    use std::fs::File;
    use std::io::BufReader;

    // Open the file in read-only mode with buffer.
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    serde_json::from_reader(reader).unwrap()
}
