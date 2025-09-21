mod prust {
    include!("prust/oneof_default_value.rs");
}

mod prost {
    include!("prost/oneof_default_value.rs");
}

conformance::fuzz!(TestOneofDefaultValue, 2000);
