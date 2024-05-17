# uneval_static

Makes [Serde](http://serde.rs) serialize your data to Rust source code that can be stored in a static variable.

### Why?

The crate that this is forked from, [uneval](https://crates.io/crates/uneval), provides type flexibility by using trait functions such as `.into()` to convert serde types to rust types. However, this means that the output code must incur some runtime cost to initialize itself, which is suboptimal on an emotional level and potentially on a performance level as well. The code output by this crate can fit into a narrower range of types, but does not require a `lazy_static` initializer.

### How to?

This crate is intended to be used from the build script. It will serialize anything you provide to it to any path you provide (or to the arbitrary [`io::Write`](https://doc.rust-lang.org/stable/std/io/trait.Write.html) implementation, or into `String`, if you want to). Then, you'll [`include!`](https://doc.rust-lang.org/stable/std/macro.include.html) the generated file wherever you want to use it. As your static variable will likely declare any references as `&'static`, you will probably want to use different type definitions in the build script than in your code. Any maps in the static variable should be declared as [phf](https://crates.io/crates/phf)

### How does it work?

See the [crate documentation](https://docs.rs/uneval) for details. In short, we use information provided by Serde to emit the code, which, when assigned to the variable of correct type, will provide all necessary conversions by using `Into` and iterators.

### Is it really that simple?

Well... not. There are several limitations.

1. All the types used in the serialized struct must be in scope on the include site. Serde doesn't provide the qualified name (i.e. path) to the serializer, only the "last" name. The probably easiest way is to use the serialized data as following:

```rust
let static VALUE: MainType = {
    use ::path::to::Type1;
    // ...and other types
    include!("path/to/file.rs")
}
```

2. As a consequence, all the types used by the serialized one must have distinct names (or they'll clash with each other).
3. Deserializer isn't implemented. This is intentional, since this crate isn't really intended for runtime usage. Well, in fact, the deserializer _is_ implemented - it's just the Rust compiler itself.
4. This serializer is intended for use with derived implementation. It may return bogus results when used with customized `Serialize`.

If you find any other case where this doesn't work, feel free to open a pull request.

### Testing

This crate uses [`trybuild`](https://crates.io/crates/trybuild) to run its tests. Each test case is output to it's own directory in `test_fixtures/`, where `{test_name}/main.rs` is compiled and run first, which creates `{test_name}/generated.rs` to be used in the compilation of `{test_name}/user.rs`, which asserts that the generated values match the inputs.

Testing data is defined in [test_fixtures/data.toml], and is in the following format:

- Section name in TOML corresponds to the name of test case. Note that there is only a single test run by cargo, with each of the test cases generated, compiled and run in multiple stages by that cargo test.
- Field `main_type` corresponds to the type which serialization is being tested.
- Field `definition` is the type definition. This will be included as-is in `{test_name}/main.rs`, and included in `{test_name}/user.rs` after applying simple string replacements to make it compatible with a static declaration. It's necessary to derive `Debug`, `Serialize` and `PartialEq` on all the types there, since these traits are used during test entry run.
- Field `value` is literally copied in two places: first, the `{test_name}/main.rs`, where the code is generated; second, in `{test_name}/user.rs`, where test checks two values for equality.
- Field `test_values` is optional, for cases where `value` cannot easily be used in `user.rs` assertions. Instead of asserting equality on the entire struct, each value in the `test_values` map generates an individual assertion.

# License

MIT
