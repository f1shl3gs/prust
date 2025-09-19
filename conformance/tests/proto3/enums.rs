mod prust {
    include!("prust/enums.rs");
}

mod prost {
    include!("prost/enums.rs");
}

#[test]
fn default() {
    let foo = prust::Foo {
        first: prust::State::default(),
        second: prust::State::default(),
    };

    assert_eq!(foo, prust::Foo::default());
}

conformance::fuzz!(Foo);
