mod r#gen;

fn main() {
    let value: r#gen::TestType = serde_json::from_reader(std::io::stdin()).unwrap();
    serde_json::to_writer(std::io::stdout(), &value).unwrap();
}
