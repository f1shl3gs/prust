mod prust {
    include!("prust/default_string_escape.rs");
}

mod prost {
    include!("prost/default_string_escape.rs");
}

#[test]
fn default() {
    let msg = prust::Person::default();
    assert_eq!(msg.name, r#"["unknown"]"#)
}

conformance::fuzz!(Person, 1000);
