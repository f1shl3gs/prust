mod prost {
    include!("prost/default_values.rs");
}

mod prust {
    #![allow(dead_code)]

    include!("prust/default_values.rs");
}

#[test]
fn required() {
    let msg = prust::TestDefaultValuesRequired::default();

    assert_eq!(msg.double_field, 1f64);
    assert_eq!(msg.float_field, 2f32);
    assert_eq!(msg.int32_field, 3i32);
    assert_eq!(msg.int64_field, 4i64);
    assert_eq!(msg.uint32_field, 5u32);
    assert_eq!(msg.uint64_field, 6u64);
    assert_eq!(msg.sint32_field, 7i32);
    assert_eq!(msg.sint64_field, 8i64);
    assert_eq!(msg.fixed32_field, 9u32);
    assert_eq!(msg.fixed64_field, 10u64);
    assert_eq!(msg.sfixed32_field, 11i32);
    assert_eq!(msg.sfixed64_field, 12i64);
    assert_eq!(msg.bool_field, true);
    assert_eq!(msg.string_field, "abc\n22".to_string());
    assert_eq!(msg.bytes_field, b"cde\n33".to_vec());
    assert_eq!(msg.enum_field, prust::EnumForDefaultValue::Two);
    assert_eq!(
        msg.enum_field_without_default,
        prust::EnumForDefaultValue::One
    );
}

conformance::fuzz!(TestDefaultValuesRequired);
