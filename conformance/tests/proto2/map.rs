#[ignore]
#[test]
fn generate() {
    const KEY_TYPES: [&str; 12] = [
        "int32", "int64", "uint32", "uint64", "sint32", "sint64", "fixed32", "fixed64", "sfixed32",
        "sfixed64", "bool", "string",
    ];

    const VALUE_TYPES: [&str; 18] = [
        "double", "float", "int32", "int64", "uint32", "uint64", "sint32", "sint64", "fixed32",
        "fixed64", "sfixed32", "sfixed64", "bool", "string", "bytes", "Empty", "Value", "State",
    ];

    let mut num = 1;
    for key in KEY_TYPES {
        for value in VALUE_TYPES {
            println!(
                "map<{}, {}> {}_{} = {};",
                key,
                value,
                key,
                value.to_lowercase(),
                num
            );
            num += 1;
        }
        println!()
    }
}

mod prust {
    #![allow(unused_variables)]
    include!("prust/map.rs");
}

mod prost {
    include!("prost/map.rs");
}

conformance::fuzz!(Data);
