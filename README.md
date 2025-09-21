# Prust

English | [中文](README_cn.md)

`prust` is a protobuf implementation for Rust. `prust` generates simple and high performance
code from `proto2` or `proto3` files.

Compare to other implementations
- highly optimized code, `prust` calculates everything when generating, so less calculating at runtime
- zero dependency, `prust` do not need that, no extra bloat
- `grpc` is supported by default (with [tonic](https://github.com/hyperium/tonic))
- less build time, since we don't need to expand proc macros
- `group` is not supported, since it is deprecated too.

### Sizes
`prust` generates structs and implements `Deserialize` and `Serialize`,
so the generated file is a little bit larger than `prost`, but still smaller 
than then expanded code.

<table>
    <thead>
        <tr>
            <th> File </th>
            <th> Crate </th>
            <th> Lines </th>
            <th> Sizes </th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td rowspan=2>proto2/data_types.proto</td>
            <td> prust </td>
            <td> 386 </td>
            <td> 16500 </td>
        </tr>
        <tr>
            <td> prost </td>
            <td> 161 </td>
            <td> 6423 </td>
        </tr>
        <tr>
            <td rowspan=2> proto3/data_types.proto </td>
            <td> prust </td>
            <td> 356 </td>
            <td> 16416 </td>
        </tr>
        <tr>
            <td> prost </td>
            <td> 165 </td>
            <td> 6603 </td>
        </tr>
    </tbody>
</table>


### Performance
With our [workload](perf/proto/perf.proto) which covers lots of cases, `prust` works very well. 
The decoding performance almost catch up `quick-protobuf`, which uses `Cow` to 
improve performance(while `prust` don't ). And the encoding performance is around 2x 
faster than `prost`. With different workload, `prust` performance will be different
too, users must verify it if you want to switch to `prust`.

```text
Decoding 6000 times
prost:   969.55 op/s,   379.43 M/s, 6.19s
quick:  1410.40 op/s,   551.96 M/s, 4.25s
prust:  1267.26 op/s,   495.94 M/s, 4.73s

Encoding 6000 times
prost:  1574.18 op/s,   616.05 M/s, 3.81s
quick:  1982.27 op/s,   775.76 M/s, 3.03s
prust:  3259.56 op/s,  1275.62 M/s, 1.84s
```

`NOTE`: `prost` seems leak memory, it takes 2.1G to finish our test, while others takes only 1.1M.

## Example
- Add `prust` to `build-dependencies`
```toml
[build-dependencies]
prust-build = 0.1
 
[dependencies]
prust = 0.1
```

- add compile functions to `build.rs` 
```rust
fn main() {
    prust_build::Config::default()
        .compile(&["/path/to/include"], &["/path/to/your.proto"])
        .unwrap();
}
```

- Include whatever the prust generated

Note: by default, generated filename is `package` field in `*.proto`, if it is not specified, 
`prust` will use the `proto`'s filename.
```rust
mod proto {
    include!(concat!(env!("OUT_DIR"), "/package.rs"));
}

use proto::Data;
use prust::{Deserialize, Serialize};

fn main() {
    let data = Data::decode(input).unwrap();

    let len = data.encoded_len();
    let buf = vec![0; len];
    data.encode(&mut buf).unwrap();
}
```

## Grpc
Grpc is supported and the generated file works with `tonic`, 
and the generated file is just like `tonic` does, so switching from `tonic`
to `prust` is very easy.
1. enable `tonic` feature of `prust`
2. fix the use path in your rust file
3. running your program or test

A running example can be found in the [conformance/tests/services/health.rs](conformance/tests/services/health.rs)

## TODO
- ~~implement default value for map's key and value, which will reduce 
encoded size and resource usage~~ it hurt the performance a little bit.
- it seems that access data via `*const u8` is better than `slice[pos]`, more test needed. 
