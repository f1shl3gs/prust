mod prust {
    include!("prust/required.rs");
}

#[test]
fn build() {
    let _ = prust::TestRequired { b: false };
    let _ = prust::TestRequiredOuter {
        inner: prust::TestRequired { b: false },
    };
}
