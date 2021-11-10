[![crates.io](https://img.shields.io/crates/v/bytes-text?style=flat-square)](https://crates.io/crates/bytes-text)
[![docs](https://img.shields.io/docsrs/bytes-text?style=flat-square)](https://docs.rs/bytes-text)
[![build status](https://img.shields.io/github/workflow/status/Cyborus04/bytes-text/rust?style=flat-square)](https://github.com/Cyborus04/bytes-text/actions/workflows/rust.yml)

# bytes-text

`Text` is to [`Bytes`]() what `String` is to `Vec<u8>`

## Example

```rust
let text = Text::from("Hello, world!");
println!("{}", text);

let hello = text.get(..5);
assert_eq!(hello, "Hello");

```

## License

This project is licensed under either of

- [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
  ([LICENSE-APACHE](LICENSE-APACHE))

- [MIT License](http://opensource.org/licenses/MIT)
  ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
