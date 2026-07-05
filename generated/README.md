# Generated schema artifacts

The local meta-signal contract convention keeps `schema/meta-signal.schema` and
checked-in Rust contract types side by side until schema-rust exposes contract
emission for meta-signal repos.

TODO when that generator is available:

```sh
SCHEMA_RUST_UPDATE_META_SIGNAL_ARTIFACTS=1 cargo check
```

The expected generated destination is `src/generated.rs`, with `src/lib.rs`
retaining hand-written documentation, re-exports, and boundary helpers.
