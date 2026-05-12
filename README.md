- [summer-rs examples](#summer-rs-examples)
- [utoipa](#spring-utoipa)
  [![crates.io](https://img.shields.io/crates/v/spring-utoipa.svg)](https://crates.io/crates/spring-utoipa)
  [![Documentation](https://docs.rs/spring-utoipa/badge.svg)](https://docs.rs/spring-utoipa):
  Utoipa offers compile time generated OpenAPI documentation for Rust.

## summer-rs examples

These crates sit next to a [summer-rs](https://github.com/summer-rs/summer-rs) checkout (sibling directory `summer-rs/`). They are **not** members of the `summer-rs` Cargo workspace; each `Cargo.toml` pins versions and uses `path` into `../summer-rs/...`.

Examples for contrib plugins live next to each crate under `<crate>/examples/`:

- [`summer-opendal/examples/opendal-example`](summer-opendal/examples/opendal-example) — run from `summer-opendal/`: `cargo run --example opendal-example --features services-fs`
- [`summer-pubsub/examples/pubsub-example`](summer-pubsub/examples/pubsub-example) — run from `summer-pubsub/`: `cargo run --example pubsub-example`
- [`summer-sa-token/examples/sa-token-example`](summer-sa-token/examples/sa-token-example) — run from `summer-sa-token/`: `cargo run --example sa-token-example`

```shell
# Layout: parent/summer-rs  and  parent/contrib-plugins
cd contrib-plugins/summer-opendal && cargo run --example opendal-example --features services-fs
```

For `summer-rs` workspace examples that reference contrib (e.g. `plugin-example`), clone this repo under `summer-rs/contrib-plugins`, or symlink `summer-rs/contrib-plugins` → `../contrib-plugins`.

## Spring utoipa

[spring-utoipa](spring-utoipa) ingegrates
[utoipa](https://github.com/juhaku/utoipa) into spring-web, providing
auto-generated OpenAPI documentation.

For specific examples, please refer to the
[simple](https://github.com/spring-rs/contrib-plugins/tree/master/spring-utoipa/examples/simple),
[with-rapidoc](https://github.com/spring-rs/contrib-plugins/tree/master/spring-utoipa/examples/with-rapidoc),
[with-redoc](https://github.com/spring-rs/contrib-plugins/tree/master/spring-utoipa/examples/with-redoc),
[with-scalar](https://github.com/spring-rs/contrib-plugins/tree/master/spring-utoipa/examples/with-scalar)
or
[with-swagger-ui](https://github.com/spring-rs/contrib-plugins/tree/master/spring-utoipa/examples/with-swagger-ui)
project.

- Run the example

```shell
cargo run --color=always --package spring-utoipa --example with-scalar --features=scalar
```
