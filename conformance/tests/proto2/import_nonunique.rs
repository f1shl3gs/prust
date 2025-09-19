mod prust {
    include!("prust/import_nonunique.rs");
}

#[test]
fn build() {
    let _ = prust::TestImportNonunque {
        n1: Some(prust::nonunique_1::Nonunique {}),
        n2: Some(prust::nonunique_2::Nonunique {}),
    };
}
