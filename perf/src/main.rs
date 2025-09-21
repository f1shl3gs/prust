mod quick;

mod prust {
    include!("prust/perf.rs");
}

mod prost {
    #![allow(dead_code)]

    // include!(concat!(env!("OUT_DIR"), "/perf.rs"));
    include!("prost/perf.rs");

    pub fn decode(input: &[u8]) {
        use ::prost::Message;
        let _data = Data::decode(&mut &input[..]).unwrap();
    }
}

use std::time::{Duration, Instant};

use quick_protobuf::{BytesReader, BytesWriter, MessageRead, MessageWrite, Writer};

const INPUT: &[u8] = include_bytes!("../proto/perf.data");

fn run(round: usize, mut f: impl FnMut()) -> Duration {
    let start = Instant::now();
    for _ in 0..round {
        f()
    }
    start.elapsed()
}

fn main() {
    let mut args = std::env::args().skip(1);
    let round = match args.next() {
        Some(arg) => arg.parse::<usize>().unwrap(),
        None => 2000,
    };

    println!("Decoding {} times", round);
    decode(round);

    println!();

    println!("Encoding {} times", round);
    encode(round);
}

fn decode(round: usize) {
    let elapsed = run(round, || prost::decode(INPUT));
    println!(
        "prost: {:>8.2} op/s, {:>8.2} M/s, {elapsed:.2?}",
        round as f64 / elapsed.as_secs_f64(),
        (round * INPUT.len()) as f64 / (1024.0 * 1024.0) / elapsed.as_secs_f64()
    );

    let elapsed = run(round, || {
        use quick_protobuf::BytesReader;
        use quick_protobuf::MessageRead;

        let mut buf = BytesReader::from_bytes(INPUT);
        let _data = quick::Data::from_reader(&mut buf, INPUT).unwrap();
    });
    println!(
        "quick: {:>8.2} op/s, {:>8.2} M/s, {elapsed:.2?}",
        round as f64 / elapsed.as_secs_f64(),
        (round * INPUT.len()) as f64 / (1024.0 * 1024.0) / elapsed.as_secs_f64()
    );

    let elapsed = run(round, || {
        use ::prust::Deserialize;

        prust::Data::decode(INPUT).unwrap();
    });
    println!(
        "prust: {:>8.2} op/s, {:>8.2} M/s, {elapsed:.2?}",
        round as f64 / elapsed.as_secs_f64(),
        (round * INPUT.len()) as f64 / (1024.0 * 1024.0) / elapsed.as_secs_f64()
    );
}

fn encode(round: usize) {
    use ::prust::{Deserialize, Serialize};

    use ::prost::Message;
    // don't know why, but it seems that prost leaks something
    let data = prost::Data::decode(INPUT).unwrap();
    let mut buf = vec![0u8; INPUT.len()];
    let elapsed = run(round, || {
        use ::prost::Message;
        data.encode(&mut buf).unwrap();
    });
    println!(
        "prost: {:>8.2} op/s, {:>8.2} M/s, {elapsed:.2?}",
        round as f64 / elapsed.as_secs_f64(),
        (round * INPUT.len()) as f64 / (1024.0 * 1024.0) / elapsed.as_secs_f64()
    );

    // ---------------------------------------------

    let mut reader = BytesReader::from_bytes(INPUT);
    let data = quick::Data::from_reader(&mut reader, INPUT).unwrap();
    let encoded_len = data.get_size();
    let mut buf = vec![0u8; INPUT.len()];
    buf.resize(encoded_len, 0);
    let elapsed = run(round, || {
        let mut writer = Writer::new(BytesWriter::new(buf.as_mut_slice()));
        data.write_message(&mut writer).unwrap();
    });
    println!(
        "quick: {:>8.2} op/s, {:>8.2} M/s, {elapsed:.2?}",
        round as f64 / elapsed.as_secs_f64(),
        (round * INPUT.len()) as f64 / (1024.0 * 1024.0) / elapsed.as_secs_f64()
    );

    // -------------------------------------------------

    let data = prust::Data::decode(INPUT).unwrap();
    let elapsed = run(round, || {
        data.encode(&mut buf).unwrap();
    });
    println!(
        "prust: {:>8.2} op/s, {:>8.2} M/s, {elapsed:.2?}",
        round as f64 / elapsed.as_secs_f64(),
        (round * INPUT.len()) as f64 / elapsed.as_secs_f64() / 1024.0 / 1024.0
    );
}

#[test]
fn encoding() {
    use ::prust::{Deserialize, Serialize};

    let data = prust::Data::decode(INPUT).unwrap();

    let len = data.encoded_len();
    let mut buf = vec![0u8; len];
    data.encode(&mut buf).unwrap();
}

#[ignore]
#[test]
fn gen_data() {
    use std::collections::BTreeMap;
    use std::iter::repeat_with;

    use crate::prust::{Complex, Data, PackedRepeats, Repeats, SelfReference, Simple, State};
    use ::prust::Serialize;
    use rand::distr::Alphanumeric;
    use rand::random_range;
    use rand::{Rng, random};

    fn random_string(n: usize) -> String {
        rand::rng()
            .sample_iter(&Alphanumeric)
            .take(n)
            .map(|u| u as char)
            .collect::<String>()
    }

    fn random_bytes(n: usize) -> Vec<u8> {
        rand::rng().sample_iter(&Alphanumeric).take(n).collect()
    }

    fn random_simple() -> Simple {
        Simple {
            key: Some(random_string(32)),
            value: Some(random::<i32>()),
        }
    }

    fn random_complex() -> Complex {
        Complex {
            double: random(),
            float: random(),
            int32: random(),
            int64: random(),
            uint32: random(),
            uint64: random(),
            sint32: random(),
            sint64: random(),
            fixed32: random(),
            fixed64: random(),
            sfixed32: random(),
            sfixed64: random(),
            bool: random(),
            small_string: random_string(16),
            large_string: random_string(256),
            small_bytes: random_bytes(16),
            large_bytes: random_bytes(256),
            state: State::try_from(random_range(0..=1)).unwrap(),
            string_int32: {
                let mut map = BTreeMap::default();
                for _ in 0..random_range(1..32) {
                    let k = random_string(random_range(1..32));
                    let v = random();
                    map.insert(k, v);
                }

                map
            },
            string_simple: {
                let mut map = BTreeMap::default();
                for _ in 0..random_range(1..32) {
                    let k = random_string(random_range(1..32));
                    let v = random_simple();
                    map.insert(k, v);
                }

                map
            },
            string_state: {
                let mut map = BTreeMap::default();
                for _ in 0..random_range(1..32) {
                    let k = random_string(random_range(1..32));
                    let v = State::try_from(random_range(0..=1)).unwrap();
                    map.insert(k, v);
                }

                map
            },
        }
    }

    fn random_self_reference() -> SelfReference {
        SelfReference {
            name: random_string(random_range(0..16)),
            value: random(),
            reference: Some(Box::new(SelfReference {
                name: random_string(random_range(0..16)),
                value: random(),
                reference: None,
            })),
        }
    }

    fn random_repeat() -> Repeats {
        Repeats {
            double: repeat_with(random).take(random_range(1..64)).collect(),
            float: repeat_with(random).take(random_range(1..64)).collect(),
            int32: repeat_with(random).take(random_range(1..64)).collect(),
            int64: repeat_with(random).take(random_range(1..64)).collect(),
            uint32: repeat_with(random).take(random_range(1..64)).collect(),
            uint64: repeat_with(random).take(random_range(1..64)).collect(),
            sint32: repeat_with(random).take(random_range(1..64)).collect(),
            sint64: repeat_with(random).take(random_range(1..64)).collect(),
            fixed32: repeat_with(random).take(random_range(1..64)).collect(),
            fixed64: repeat_with(random).take(random_range(1..64)).collect(),
            sfixed32: repeat_with(random).take(random_range(1..64)).collect(),
            sfixed64: repeat_with(random).take(random_range(1..64)).collect(),
            bool: repeat_with(random).take(random_range(1..64)).collect(),
            string: repeat_with(|| random_string(random_range(1..128)))
                .take(random_range(1..64))
                .collect(),
            bytes: repeat_with(|| Vec::from(random_string(random_range(1..32))))
                .take(random_range(1..64))
                .collect(),
        }
    }

    fn random_packed_repeat() -> PackedRepeats {
        PackedRepeats {
            double: repeat_with(random).take(random_range(1..64)).collect(),
            float: repeat_with(random).take(random_range(1..64)).collect(),
            int32: repeat_with(random).take(random_range(1..64)).collect(),
            int64: repeat_with(random).take(random_range(1..64)).collect(),
            uint32: repeat_with(random).take(random_range(1..64)).collect(),
            uint64: repeat_with(random).take(random_range(1..64)).collect(),
            sint32: repeat_with(random).take(random_range(1..64)).collect(),
            sint64: repeat_with(random).take(random_range(1..64)).collect(),
            fixed32: repeat_with(random).take(random_range(1..64)).collect(),
            fixed64: repeat_with(random).take(random_range(1..64)).collect(),
            sfixed32: repeat_with(random).take(random_range(1..64)).collect(),
            sfixed64: repeat_with(random).take(random_range(1..64)).collect(),
            bool: repeat_with(random).take(random_range(1..64)).collect(),
        }
    }

    let data = Data {
        state: TryFrom::try_from(random_range(0..=1)).unwrap(),
        states: repeat_with(|| State::try_from(random_range(0..=1)).unwrap())
            .take(random_range(0..64))
            .collect(),
        simple: random_simple(),
        simples: repeat_with(random_simple)
            .take(random_range(0..64))
            .collect(),
        complex: random_complex(),
        complexes: repeat_with(random_complex)
            .take(random_range(0..64))
            .collect(),
        self_reference: random_self_reference(),
        self_references: repeat_with(random_self_reference)
            .take(random_range(0..64))
            .collect(),
        repeat: random_repeat(),
        repeats: repeat_with(random_repeat)
            .take(random_range(0..64))
            .collect(),
        packed_repeat: random_packed_repeat(),
        packed_repeats: repeat_with(random_packed_repeat)
            .take(random_range(0..64))
            .collect(),
    };

    let len = data.encoded_len();
    let mut buf = vec![0; len];
    let written = data.encode(&mut buf).unwrap();
    println!("{}", written);

    std::fs::write("ng.data", &buf[..written]).unwrap();
}
