use prust_build::Config;

fn help() {
    println!("A converter from proto files to rust files");
    println!("USAGE:");
    println!("  prust [OPTIONS] [FLAGS] [FILES]");
    println!();
    println!("FLAGS:");
    println!("  -h, --help           Prints help information");
    println!();
    println!("OPTIONS:");
    println!("  -i, --include        Path to search for imported protobuf files");
    println!("  -d                   Output directory for generated files");
    println!("  -o                   Generated file name");
}

fn handle_error(code: i32, msg: &str) {
    println!("{}", msg);
    std::process::exit(code);
}

fn main() {
    let mut args = std::env::args().skip(1);
    let mut includes = Vec::new();
    let mut protos = Vec::new();
    let mut output = String::new();

    while let Some(value) = args.next() {
        match value.as_str() {
            "-i" | "--include" => {
                let Some(value) = args.next() else {
                    handle_error(1, "include value not provided");
                    return;
                };

                includes.push(value);
            }
            "-d" => {
                let Some(value) = args.next() else {
                    handle_error(1, "output directory not provided");
                    panic!("-d needs a value");
                };

                output = value;
            }
            "-o" => {
                let Some(value) = args.next() else {
                    handle_error(1, "filename not provided");
                    return;
                };

                output = value;
            }
            "-h" | "--help" => {
                help();
                return;
            }
            _ => protos.push(value),
        }
    }

    if protos.is_empty() {
        panic!("`protos` must be specified");
    }

    println!("includes: {:?}", includes);
    println!("protos: {:?}", protos);

    Config::default()
        .output(output)
        .compile(&includes, &protos)
        .unwrap();
}
