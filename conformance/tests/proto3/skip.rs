#[test]
fn check_implementation() {
    let generated = include_str!("prust/skip.rs");

    assert!(!generated.contains("impl Deserialize for NoDeserialize {"));
    assert!(generated.contains("impl Serialize for NoDeserialize {"));

    assert!(generated.contains("impl Deserialize for NoSerialize {"));
    assert!(!generated.contains("impl Serialize for NoSerialize {"));

    assert!(!generated.contains("impl Deserialize for NoDeserializeAndSerialize {"));
    assert!(!generated.contains("impl Serialize for NoDeserializeAndSerialize {"));
}
