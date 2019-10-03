extern crate hackatom;

#[test]
fn test_coin() {
    let c = hackatom::types::coin("123", "tokens");
    assert_eq!(c.len(), 1);
    assert_eq!(c.get(0).unwrap().amount, "123");
}