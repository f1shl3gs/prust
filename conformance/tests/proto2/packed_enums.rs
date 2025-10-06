mod prust {
    include!("prust/packed_enums.rs");
}

mod prost {
    include!("prost/packed_enums.rs");
}

conformance::fuzz!(Data);

#[test]
fn manually() {
    use self::prust::{Data, LargeState, State};
    use ::prust::Deserialize;

    for (data, expected) in [
        (
            [1 << 3 | 0, 1, 1 << 3 | 0, 0].as_slice(),
            Data {
                non_packed: vec![State::On, State::Off],
                ..Default::default()
            },
        ),
        (
            [2 << 3 | 2, 2, 1, 0].as_slice(),
            Data {
                packed: vec![State::On, State::Off],
                ..Default::default()
            },
        ),
        (
            [3 << 3 | 0, 128, 1, 3 << 3 | 0, 0].as_slice(),
            Data {
                large_non_packed: vec![LargeState::Yes, LargeState::No],
                ..Default::default()
            },
        ),
        (
            [4 << 3 | 2, 3, 128, 1, 0].as_slice(),
            Data {
                large_packed: vec![LargeState::Yes, LargeState::No],
                ..Default::default()
            },
        ),
    ] {
        let got = Data::decode(data).unwrap();
        assert_eq!(got, expected);
    }
}
