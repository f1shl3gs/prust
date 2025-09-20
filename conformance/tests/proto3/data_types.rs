use ::prust::Deserialize;

mod prust {
    #![allow(dead_code, unused_imports)]

    include!("prust/data_types.rs");
}

pub mod a {
    pub mod b {
        include!("prost/a.b.rs");
    }
}

mod prost {
    include!("prost/data_types.rs");
}

conformance::fuzz!(FooMessage);

#[test]
fn decode() {
    use ::prost::Message;

    let input = &[
        8, 193, 225, 163, 226, 4, 16, 228, 178, 128, 168, 204, 237, 216, 254, 67, 24, 197, 199,
        202, 139, 10, 32, 189, 128, 133, 165, 205, 181, 152, 243, 176, 1, 40, 150, 178, 134, 225,
        7, 48, 220, 189, 199, 188, 202, 197, 192, 194, 241, 1, 73, 20, 81, 113, 159, 22, 250, 73,
        81, 81, 47, 243, 57, 101, 202, 96, 242, 41, 93, 159, 113, 230, 150, 101, 47, 115, 118, 70,
        105, 46, 169, 114, 210, 74, 7, 14, 167, 117, 179, 160, 141, 227, 130, 1, 1, 97, 162, 1, 5,
        188, 155, 229, 231, 5, 170, 1, 4, 149, 63, 5, 60, 194, 1, 0, 200, 1, 2, 210, 1, 8, 10, 0,
        16, 232, 189, 193, 227, 3, 210, 1, 11, 10, 3, 6, 204, 162, 16, 204, 139, 142, 148, 1, 242,
        1, 0, 242, 1, 4, 27, 86, 197, 154, 242, 1, 0,
    ];

    let out = prost::FooMessage::decode(input.as_slice()).unwrap();
    println!("out: {:?}", out.f_map);
    // println!("{}", out.encoded_len());

    println!("-------------------------");

    let out = prust::FooMessage::decode(input).unwrap();
    println!("out: {:?}", out.f_map);
    // println!("{}", out.encoded_len());
}
