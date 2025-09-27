#![allow(unused_imports)]

include!("prust/presence.rs");

#[test]
fn test_proto3_presence() {
    let msg = A {
        b: Some(42),
        foo: Some(a::Foo::C(13)),
    };

    conformance::check_message(&msg);
}
