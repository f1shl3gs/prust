fn main() {
    prost_build::Config::default()
        .out_dir("src/prost")
        .compile_protos(&["proto/perf.proto"], &["proto"])
        .unwrap();
    prust_build::Config::default()
        .message_attribute("", "#[derive(arbitrary::Arbitrary, serde::Serialize)]")
        .enum_attribute("", "#[derive(arbitrary::Arbitrary, serde::Serialize)]")
        .output("src")
        .filename("prust")
        .compile(&["proto/"], &["proto/perf.proto"])
        .unwrap();

    prost_build::Config::default()
        // .out_dir("src/prost")
        .compile_protos(&["prw/remote.proto"], &["prw"])
        .unwrap();
    prust_build::Config::default()
        .output("prw")
        .compile(&["prw"], &["prw/remote.proto"])
        .unwrap();
}
