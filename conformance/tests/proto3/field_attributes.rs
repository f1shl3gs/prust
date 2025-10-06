#[test]
fn check_attributes() {
    let generated = include_str!("prust/field_attributes.rs");

    assert!(generated.contains("#[serde(skip_serializing)]\n    pub first: i32"));
}
