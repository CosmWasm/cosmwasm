
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub enum SomeEnum {
    Field1,
    Field2(u32, u32),
    Field3 {
        a: String,
        b: u32
    },
    Field4(Box<SomeEnum>),
    Field5 { a: Box<SomeEnum> },
}

#[derive(Serialize, Deserialize)]
pub struct UnitStructure;

#[derive(Serialize, Deserialize)]
pub struct TupleStructure(u32, String, u128);

#[derive(Serialize, Deserialize)]
pub struct NamedStructure {
    a: String,
    b: u8,
    c: SomeEnum
}


#[cfg(not(feature = "deserialize"))]
fn main() {
    println!("{}", serde_json::to_string(&SomeEnum::Field1).unwrap());
    println!("{}", serde_json::to_string(&SomeEnum::Field2(10, 23)).unwrap());
    println!("{}", serde_json::to_string(&SomeEnum::Field3 {a: "sdf".to_string(), b: 12}).unwrap());
    println!("{}", serde_json::to_string(&SomeEnum::Field4(Box::new(SomeEnum::Field1))).unwrap());
    println!("{}", serde_json::to_string(&SomeEnum::Field5 { a: Box::new(SomeEnum::Field1) }).unwrap());
    println!("{}", serde_json::to_string(&UnitStructure {}).unwrap());
    println!("{}", serde_json::to_string(&TupleStructure(10, "aasdf".to_string(), 2)).unwrap());
    println!("{}", serde_json::to_string(&NamedStructure {a: "awer".to_string(), b: 4, c: SomeEnum::Field1}).unwrap());
}

#[cfg(feature = "deserialize")]
fn main() {
    use std::io::BufRead;
    for (index, line) in std::io::BufReader::new(std::io::stdin()).lines().enumerate() {
        let line = line.unwrap();
        println!("{line}");
        if index < 5 {
            let _: SomeEnum = serde_json::from_str(&line).unwrap();
        } else if index == 5 {
            let _: UnitStructure = serde_json::from_str(&line).unwrap();
        } else if index == 6 {
            let _: TupleStructure = serde_json::from_str(&line).unwrap();
        } else {
            let _: NamedStructure = serde_json::from_str(&line).unwrap();
        }
    }
}
