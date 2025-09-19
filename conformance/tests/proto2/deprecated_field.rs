mod prust {
    include!("prust/deprecated_field.rs");
}

mod prost {
    include!("prost/deprecated_field.rs");
}

#[test]
fn no_deprecated_field() {
    let _t = prust::Test {
        not_outdated: "foo".to_string(),
    };
}

conformance::fuzz!(Test);
