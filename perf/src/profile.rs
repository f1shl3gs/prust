mod proto {
    include!("prust.rs");
}

const INPUT: &[u8; 410357] = include_bytes!("../proto/perf.data");

fn main() {
    let mut args = std::env::args().skip(1);
    let round = match args.next() {
        Some(arg) => arg.parse::<usize>().unwrap(),
        None => 2000,
    };

    profile("encoding", round, encode);
    profile("decoding", round, decode);
}

fn profile(name: &str, round: usize, f: impl FnOnce(usize)) {
    let guard = pprof::ProfilerGuard::new(5000).unwrap();

    f(round);

    if let Ok(report) = guard.report().build() {
        let file = std::fs::File::create(format!("{}.svg", name)).unwrap();
        report.flamegraph(file).unwrap();
    };
}

fn encode(round: usize) {
    use ::prust::{Deserialize, Serialize};

    let data = proto::Data::decode(INPUT).unwrap();
    let mut buf = vec![0u8; data.encoded_len()];

    for _ in 0..round {
        data.encode(&mut buf).unwrap();
    }
}

fn decode(round: usize) {
    use ::prust::Deserialize;

    for _ in 0..round {
        let _ = proto::Data::decode(INPUT).unwrap();
    }
}
