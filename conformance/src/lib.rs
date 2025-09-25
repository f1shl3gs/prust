#[macro_export]
macro_rules! fuzz {
    ($typ:ident) => {
        $crate::fuzz! { $typ, 2000 }
    };

    ($typ:ident, $round: expr) => {
        $crate::fuzz! { $typ, $round, 512 }
    };

    ($typ:ident, $round:expr, $seed_len:expr) => {
        ::paste::paste! {
            #[test]
            fn [< fuzz_ $typ >]() {
                use ::arbitrary::Arbitrary;
                use ::prost::Message;
                use ::prust::{Deserialize, Serialize};

                for _ in 0..$round {
                    let data = rand::random::<[u8; $seed_len]>();
                    let mut unstructured = arbitrary::Unstructured::new(&data);

                    let orig = prust::$typ::arbitrary(&mut unstructured).unwrap();
                    let calculated = orig.encoded_len();
                    let mut buf = vec![0u8; calculated];
                    let written = match orig.encode(&mut buf) {
                        Ok(written) => written,
                        Err(err) => {
                            panic!("encode original random data failed, {}", err);
                        }
                    };

                    assert_eq!(calculated, written, "calculated length is not equal to written\n{:?}", orig);

                    let out = match prust::$typ::decode(&buf[..written]) {
                        Ok(decoded) => decoded,
                        Err(err) => {
                            println!("buf: {:?}", &buf[..written]);
                            panic!("decode prust encoded data failed, {}", err);
                        }
                    };

                    // f64 and f32 cannot be compared
                    let a = serde_json::to_string(&orig).unwrap();
                    let b = serde_json::to_string(&out).unwrap();
                    let a = serde_json::from_str::<serde_json::Value>(&a).unwrap();
                    let b = serde_json::from_str::<serde_json::Value>(&b).unwrap();
                    assert_eq!(a, b, "decoded is not equal to the original one");

                    let out = match prost::$typ::decode(buf.as_slice()) {
                        Ok(out) => out,
                        Err(err) => {
                            println!("orig: {:#?}", orig);
                            println!("data: {:?}", buf);
                            panic!("{err}")
                        }
                    };
                }
            }
        }
    };
}
