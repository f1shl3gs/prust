# Prust

`prust` is a protobuf implementation for Rust. `prust` generates simple and high performance
code from `proto2` or `proto3` files.

Compare to other implementations
- highly optimized code, `prust` calculates everything when generating, so less calculating at runtime
- zero dependency, `prust` do not need that, no extra bloat
- less build time, since we don't need to expand proc macros
- `group` is not supported, since it is deprecated too.

### Sizes
`prust` generates structs and implements `Deserialize` and `Serialize`,
so the generated file is a little bit larger than `prost` (which uses 
procedure macros to implement `prost::Message` and so on).

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
With our [workload](perf/proto/perf.proto) which covers lots of cases, 
`prust` works very well. And, be aware, difference workloads impacts the performance a lot.

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

`NOTE`: `prust` does not support Cow, so the decoding performance is not as good as `quick-protobuf`.

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

## TODO
- implement default value for map's key and value, which will reduce 
encoded size and resource usage 
