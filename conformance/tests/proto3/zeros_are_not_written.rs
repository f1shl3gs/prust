mod prust {
    include!("prust/zeros_are_not_written.rs");
}

mod prost {
    include!("prost/zeros_are_not_written.rs");
}

#[test]
fn zeros_are_not_written() {
    use ::prust::*;
    use prust::*;

    let mut m = ZerosAreNotWritten::default();
    m.bool_field = false;
    m.enum_field = EnumDescriptor::Undefined;
    m.fixed32_field = 0;

    let len = m.encoded_len();
    let mut buf = vec![0u8; len];
    let written = m.encode(&mut buf).unwrap();
    assert_eq!(written, len);
    assert_eq!(m.encoded_len(), 0);
}

conformance::fuzz!(ZerosAreNotWritten);
