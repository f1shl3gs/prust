mod prust {
    include!("prust/enums.rs");
}

mod prost {
    include!("prost/enums.rs");
}

#[test]
fn default() {
    let foo = prust::Foo {
        first: None,
        second: prust::State::default(),
    };

    assert_eq!(foo, prust::Foo::default());
}

conformance::fuzz!(Foo);
