mod prust {
    include!("prust/boxed_field.rs");
}

mod prost {
    include!("prost/boxed_field.rs");
}

#[test]
fn build() {
    let _foo = prust::Foo {
        bar: Some(prust::Bar {
            foo: "".to_string(),
            foo1: Some("".to_string()),
        }),
        foo: Some(Box::new(prust::Foo {
            bar: None,
            foo: None,
            foos: vec![],
        })),
        foos: vec![prust::Foo {
            bar: None,
            foo: None,
            foos: vec![],
        }],
    };
}

// if the seed too big, this will reach the prost recursive limit
conformance::fuzz!(Foo, 1000, 128);
