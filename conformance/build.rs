use std::path::PathBuf;

fn main() {
    // proto2
    compile(
        "tests/proto2",
        &[
            "data_types.proto",
            "default_enum_value.proto",
            "default_string_escape.proto",
            "default_values.proto",
            "deprecated_field.proto",
            "oneof_default_value.proto",
            "packed_enums.proto",
            "map.proto",
            "import_nested.proto",
            "import_nonunique.proto",
            "import_pkg_nested.proto",
            "import_root.proto",
            "oneof_default_value.proto",
            "required.proto",
            "sanitize-file-name.proto",
            "special~characters file{name}.proto",
        ],
    );

    // proto3
    compile(
        "tests/proto3",
        &[
            "data_types.proto",
            "boxed_field.proto",
            "enums.proto",
            "keyword_enum_variant.proto",
            "optional_empty.proto",
            "optional.proto",
            "presence.proto",
            "zeros_are_not_written.proto",
        ],
    );

    // services
    compile("tests/services", &["health.proto", "example.proto"]);

    // custom map
    prust_build::Config::default()
        .btree_map(&["different_map_type.Data.btreemap"])
        .btree_map(&["different_map_type.Data.Inner.btreemap"])
        .output("tests/proto3/prust")
        .compile(&[], &["tests/proto3/different_map_type.proto"])
        .unwrap();

    tonic_prost_build::configure()
        .out_dir("tests/services/tonic")
        .compile_protos(&["tests/services/health.proto"], &["tests/services"])
        .unwrap();
    tonic_prost_build::configure()
        .out_dir("tests/services/tonic")
        .compile_protos(&["tests/services/example.proto"], &["tests/services"])
        .unwrap();
}

fn compile(root: &str, protos: &[&str]) {
    let root = PathBuf::from(root);
    let protos = protos.iter().map(|p| root.join(p)).collect::<Vec<_>>();

    std::fs::create_dir_all(root.join("prost")).unwrap();
    std::fs::create_dir_all(root.join("prust")).unwrap();

    prust_build::Config::default()
        .message_attribute(
            "",
            "#[derive(arbitrary::Arbitrary, serde::Serialize, PartialEq)]",
        )
        .enum_attribute("", "#[derive(arbitrary::Arbitrary, serde::Serialize)]")
        .oneof_attribute(
            "",
            "#[derive(arbitrary::Arbitrary, serde::Serialize, PartialEq)]",
        )
        .output(root.join("prust"))
        .compile(&[root.clone()], protos.as_slice())
        .unwrap();

    prost_build::Config::default()
        .out_dir(root.join("prost"))
        .compile_protos(protos.as_slice(), &[root])
        .unwrap();
}
