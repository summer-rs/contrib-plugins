- [summer-rs examples](#summer-rs-examples)
- [utoipa](#spring-utoipa)
  [![crates.io](https://img.shields.io/crates/v/spring-utoipa.svg)](https://crates.io/crates/spring-utoipa)
  [![Documentation](https://docs.rs/spring-utoipa/badge.svg)](https://docs.rs/spring-utoipa):
  Utoipa offers compile time generated OpenAPI documentation for Rust.

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
