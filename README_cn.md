# Prust

[English](README.md) | 中文

`prust` 是一个 Protobuf 的 Rust 实现。 `prust` 可以生成简单并且高性能的代码， `proto2` 和 `proto3` 都是受支持的。

和其他实现的对比：
- 高度优化的代码，`prust` 会在生成阶段计算好所有的数据，运行时的开销将会降到最低
- 零依赖，`prust` 不依赖任何第三方library，可以降低最终二进制文件的大小
- 通过生成 [tonic](https://github.com/hyperium/tonic) 兼容的代码，以支持`grpc`
- 由于没有过程宏，编译速度可以得到一定的提升
- 不支持 `group`， `protobuf` 也已经弃用 `group`了
- 不再需要安装 `protoc`， `prust-build` 解析 `*.proto`

### 文件大小
`prust` 生成的文件不仅包含了相应的 `struct` / `enum` 还包含了相应的 `Deserialize` 和 `Serialize` 实现，
因此生成的文件将会比 `prost` 生成的文件大一些， 但是依旧比 `prost` 过程宏生成的代码小。 

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
            <td> 423 </td>
            <td> 18054 </td>
        </tr>
        <tr>
            <td> prost </td>
            <td> 165 </td>
            <td> 6571 </td>
        </tr>
        <tr>
            <td rowspan=2> proto3/data_types.proto </td>
            <td> prust </td>
            <td> 383 </td>
            <td> 17540 </td>
        </tr>
        <tr>
            <td> prost </td>
            <td> 165 </td>
            <td> 6603 </td>
        </tr>
    </tbody>
</table>

### 性能对比
在测试的 [负载](perf/proto/perf.proto) 下，`prust` 性能表现非常优异，
解码性能甚至快要追上使用了 Cow 来优化解码性能的 `quick-protobuf`, 
编码性能大约是`prost`的两倍。在不同负载下，`prust` 性能表现也会不同的，如果你打算
切换至`prust`，你需要自行测试验证。

```text
Decoding 6000 times
prost:   964.93 op/s,   377.62 M/s, 6.22s
quick:  1426.05 op/s,   558.08 M/s, 4.21s
prust:  1235.72 op/s,   483.60 M/s, 4.86s

Encoding 6000 times
prost:  1577.11 op/s,   617.20 M/s, 3.80s
quick:  1956.55 op/s,   765.69 M/s, 3.07s
prust:  3375.67 op/s,  1321.06 M/s, 1.78s
```

`NOTE`： `prost` 似乎发生了内存泄漏，在运行时使用了 2.1G 内存，其它的只使用约 1.1M。

## 示例
- 添加 `prust` 到 `build-dependencies`
```toml
[build-dependencies]
prust-build = 0.1
 
[dependencies]
prust = 0.1
```

- 编译对应文件
```rust
fn main() {
    prust_build::Config::default()
        .compile(&["/path/to/include"], &["/path/to/your.proto"])
        .unwrap();
}
```

- 开始使用
NOTE： 默认情况下，`prust` 将使用 `package` 作为文件名，如果`*.proto`中没有指定则使用proto文件的名称。
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
`prust` 会根据 service 生成对应的并且兼容 [tonic](https://github.com/hyperium/tonic) 的代码来支持 Grpc，所以从`prost`切换至
`prust` 将会非常容易。
1. 启用 `prust` 的 `tonic` feature。
2. 修改对应的引用代码
3. 测试运行

完整实例可以在 [conformance/tests/services/health.rs](conformance/tests/services/health.rs) 找到。

## TODO
- ~~实现 map 的 key/value 的默认值检查，以生成更小的二进制数据，降低CPU资源消耗~~ 编/解码性能有所下降
- 使用 `*const u8` 似乎比 `slice[pos]` 性能更好，还需要更多的测试验证。
- 支持 [Well-Known Types](https://protobuf.dev/reference/protobuf/google.protobuf/)
- `prust` 不能处理循环引用的 message
