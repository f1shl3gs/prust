include!("prust/keyword_enum_variant.rs");

#[test]
fn build() {
    let _ = Feeding::Assisted;
    let _ = Feeding::Self_;
    let _ = Feeding::Else;
    let _ = Feeding::Error;
    let _ = Feeding::Gen;

    let _ = Grooming::Assisted;
    let _ = Grooming::Self_;
    let _ = Grooming::Else;

    let _ = Number::Number1;
}
