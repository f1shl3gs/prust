use ::prust::Deserialize;
use quick_protobuf::BytesReader;
use std::time::Instant;

mod prust {
    #![allow(unused_variables)]
    #![allow(dead_code)]

    include!("prometheus.rs");
}

mod prost {
    #![allow(dead_code)]

    include!(concat!(env!("OUT_DIR"), "/prometheus.rs"));
}

mod quick;

fn main() {
    let mut args = std::env::args().skip(1);
    let round = match args.next() {
        Some(arg) => arg.parse::<usize>().unwrap(),
        None => 500,
    };

    let input = include_bytes!("1709380533560664458.data");
    // let input = include_bytes!("1709380533705807779.data");

    // let guard = pprof::ProfilerGuard::new(5000).unwrap();

    let start = Instant::now();
    for _ in 0..round {
        let _data = prust::WriteRequest::decode(input).unwrap();
    }
    let elapsed = start.elapsed();

    // if let Ok(report) = guard.report().build() {
    //     let file = std::fs::File::create("flamegraph.svg").unwrap();
    //     report.flamegraph(file).unwrap();
    // };

    println!("Decode {round} times");
    println!(
        "prust: {:>8.2} op/s, {:>8.2} M/s, {:>8.2} ms/op, {elapsed:.2?}",
        round as f64 / elapsed.as_secs_f64(),
        (input.len() * round) as f64 / elapsed.as_secs_f64() / 1024.0 / 1024.0,
        elapsed.as_secs_f64() * 1000.0 / round as f64
    );

    // ----------------------

    let start = Instant::now();
    for _ in 0..round {
        use ::prost::Message;
        prost::WriteRequest::decode(&input[..]).unwrap();
    }
    let elapsed = start.elapsed();
    println!(
        "prost: {:>8.2} op/s, {:>8.2} M/s, {:>8.2} ms/op, {elapsed:.2?}",
        round as f64 / elapsed.as_secs_f64(),
        (input.len() * round) as f64 / elapsed.as_secs_f64() / 1024.0 / 1024.0,
        elapsed.as_secs_f64() * 1000.0 / round as f64
    );

    // ----------------------

    let start = Instant::now();
    for _ in 0..round {
        use quick_protobuf::MessageRead;

        let mut reader = BytesReader::from_bytes(input.as_slice());
        quick::WriteRequest::from_reader(&mut reader, input).unwrap();
    }
    let elapsed = start.elapsed();
    println!(
        "quick: {:>8.2} op/s, {:>8.2} M/s, {:>8.2} ms/op, {elapsed:.2?}",
        round as f64 / elapsed.as_secs_f64(),
        (input.len() * round) as f64 / elapsed.as_secs_f64() / 1024.0 / 1024.0,
        elapsed.as_secs_f64() * 1000.0 / round as f64
    );
}
