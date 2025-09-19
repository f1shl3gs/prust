mod prust {
    #![allow(dead_code)]

    include!("prust/foo_bar.rs");
}

#[test]
fn build() {
    let _foo = prust::ContainsImportedNested {
        m: Some(prust::foo::baz::container_for_nested::NestedMessage {}),
        e: Some(prust::foo::baz::container_for_nested::NestedEnum::Red),
    };
}
