mod minus {
    include!("prust/sanitize_file_name.rs");
}

mod special {
    include!("prust/special.rs");
}

#[test]
fn sanitize_filepath() {
    let _ = minus::FooBar;
    let _ = special::Outer;
}
