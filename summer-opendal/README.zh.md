
[![crates.io](https://img.shields.io/crates/v/summer-opendal.svg)](https://crates.io/crates/summer-opendal)
[![Documentation](https://docs.rs/summer-opendal/badge.svg)](https://docs.rs/summer-opendal)

## 依赖

```toml
summer-opendal = { version = "<version>" }
```

## 配置

```toml
[opendal]
scheme = "fs"                # OpenDAL支持的服务
options = { root = "/tmp" }  # 服务配置项，不同的scheme有不同的配置项
layers = []                  # Layer是拦截操作的机制
```

Layer的相关配置, 可参看[这个文档](https://docs.rs/opendal/latest/opendal/layers/index.html)

## Components

配置完以上配置项后，插件会自动注册一个 [`Op`](https://docs.rs/summer-opendal/latest/summer_opendal/type.Op.html) 客户端。该对象是 [`opendal::Operator`](https://docs.rs/opendal/latest/opendal/struct.Operator.html) 的别名。

```rust
pub type Op = Operator;
```

完整示例见 [`summer-opendal/examples/opendal-example`](https://github.com/spring-rs/contrib-plugins/tree/master/summer-opendal/examples/opendal-example)。在本 crate 目录下执行：

```shell
cargo run --example opendal-example --features services-fs
```
