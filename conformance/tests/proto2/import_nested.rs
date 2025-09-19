mod prust {
    include!("prust/import_nested.rs");
}

#[test]
fn msg() {
    let _m = prust::ContainsImportedNested {
        m: Some(prust::import_nested_imported::container_for_nested::NestedMessage {}),
        e: Some(prust::import_nested_imported::container_for_nested::NestedEnum::Red),
        c: Some(prust::import_nested_imported::ContainerForNested { num: Some(1) }),
        top: None,
        second: None,
        inner: None,
    };
}
