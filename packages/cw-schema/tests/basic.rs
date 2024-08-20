use cw_schema::{EnumCase, Node, NodeType, Schema, SchemaV1, StructProperty};

#[test]
fn roundtrip() {
    /*
    let schema_struct = Schema::V1(SchemaV1 {
        root: Node {
            name: Some("root".into()),
            description: Some("root node".into()),
            optional: false,
            value: NodeContent::Concrete(NodeType::Object {
                properties: vec![
                    StructProperty {
                        name: "foo".into(),
                        description: Some("foo property".into()),
                        value: Node {
                            name: None,
                            description: None,
                            optional: false,
                            value: NodeContent::Concrete(NodeType::String),
                        },
                    },
                    StructProperty {
                        name: "bar".into(),
                        description: Some("bar property".into()),
                        value: Node {
                            name: None,
                            description: None,
                            optional: false,
                            value: NodeContent::Concrete(NodeType::Integer {
                                signed: false,
                                precision: 64,
                            }),
                        },
                    },
                    StructProperty {
                        name: "union".into(),
                        description: Some("union property".into()),
                        value: Node {
                            name: None,
                            description: None,
                            optional: false,
                            value: NodeContent::OneOf {
                                one_of: vec![
                                    Node {
                                        name: None,
                                        description: None,
                                        optional: true,
                                        value: NodeContent::Concrete(NodeType::String),
                                    },
                                    Node {
                                        name: None,
                                        description: None,
                                        optional: false,
                                        value: NodeContent::Concrete(NodeType::Integer {
                                            signed: true,
                                            precision: 128,
                                        }),
                                    },
                                ],
                            },
                        },
                    },
                    StructProperty {
                        name: "tagged_union".into(),
                        description: Some("tagged union property".into()),
                        value: Node {
                            name: None,
                            description: None,
                            optional: false,
                            value: NodeContent::Concrete(NodeType::Enum {
                                discriminator: Some("type".into()),
                                cases: vec![
                                    EnumCase {
                                        name: "string".into(),
                                        description: Some("string case".into()),
                                        discriminator_value: None,
                                        value: Some(Node {
                                            name: None,
                                            description: None,
                                            optional: true,
                                            value: NodeContent::Concrete(NodeType::String),
                                        }),
                                    },
                                    EnumCase {
                                        name: "number".into(),
                                        description: Some("number case".into()),
                                        discriminator_value: None,
                                        value: Some(Node {
                                            name: None,
                                            description: None,
                                            optional: false,
                                            value: NodeContent::Concrete(NodeType::Integer {
                                                signed: false,
                                                precision: 8,
                                            }),
                                        }),
                                    },
                                ],
                            }),
                        },
                    },
                ],
            }),
        },
    });

    let schema = serde_json::to_string(&schema_struct).unwrap();

    pretty_assertions::assert_eq!(
        schema_struct,
        serde_json::from_str::<Schema>(&schema).unwrap()
    );
    */
}

#[test]
fn can_decode_example() {
    let example = include_str!("example.json");
    let _: Schema = serde_json::from_str(example).unwrap();
}

#[test]
fn snapshot_jsonschema() {
    let schema = schemars::schema_for!(Schema);
    insta::assert_json_snapshot!(schema);
}
