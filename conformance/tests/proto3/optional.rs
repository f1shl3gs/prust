mod proto {
    #![allow(unused_imports)]

    include!("prust/optional.rs");
}

#[test]
fn build() {
    let _ = proto::TestOptionalProto3 {
        non_optional: 1,
        one: Some(proto::test_optional_proto3::One::OneField1("1".to_string())),
        iii: Some(1),
        sss: Some("foo".to_string()),
    };
}
