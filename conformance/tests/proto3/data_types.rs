use std::collections::HashMap;

use ::prust::{Deserialize, sizeof_len};

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

// validate::fuzz!(FooMessage);

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

pub fn sizeof_varint(v: u64) -> usize {
    match v {
        0x0..=0x7F => 1,
        0x80..=0x3FFF => 2,
        0x4000..=0x1FFFFF => 3,
        0x200000..=0xFFFFFFF => 4,
        0x10000000..=0x7FFFFFFFF => 5,
        0x0800000000..=0x3FFFFFFFFFF => 6,
        0x040000000000..=0x1FFFFFFFFFFFF => 7,
        0x02000000000000..=0xFFFFFFFFFFFFFF => 8,
        0x0100000000000000..=0x7FFFFFFFFFFFFFFF => 9,
        _ => 10,
    }
}

// prost will not serialize map key or value if they are default value
#[ignore]
#[test]
fn cs() {
    use ::arbitrary::{Arbitrary, Unstructured};
    use ::prost::Message;
    use ::prust::Serialize;

    for _ in 0..1000 {
        let data = rand::random::<[u8; 128]>();
        let mut unstructed = Unstructured::new(&data);

        let orig = prust::FooMessage::arbitrary(&mut unstructed).unwrap();
        let calculated = orig.encoded_len();
        let mut buf = vec![0u8; calculated];
        let written = orig.encode(&mut buf).unwrap();
        assert_eq!(
            calculated, written,
            "encoded_len is not equal to the written"
        );

        let out = prost::FooMessage::decode(&buf[..written]).unwrap();
        let calculated = out.encoded_len();
        assert_eq!(
            written,
            calculated,
            "prost calculated size is not equal to the prust written\n{:?}\n{:?}\n{:?}",
            orig,
            out,
            &buf[..written]
        );

        let buf = out.encode_to_vec();
        assert_eq!(
            buf.len(),
            calculated,
            "prost encoded len is not equal to the written"
        );
    }
}
