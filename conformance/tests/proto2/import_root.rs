mod prust {
    include!("prust/import_root.rs");
}

#[test]
fn build() {
    let _ = prust::ContainsImported {
        imported_message: Some(prust::import_root_imported::ImportedMessage),
        imported_enum: Some(prust::import_root_imported::ImportedEnum::Something),
    };
}
