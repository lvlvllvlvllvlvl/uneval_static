# uneval_static

Makes [Serde](http://serde.rs) serialize your data to Rust source code that can be stored in a static variable.

### How?

generate the data in `build.rs`:

```rust
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct Unit(());

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct Input {
    string: String,
    vector: Vec<i32>,
    reference: Option<Box<Input>>,
    map: HashMap<String, Unit>,
}

fn main() {
    let input = Input::deserialize(json!({
        "string": "test",
        "vector": [1,2,3],
        "reference": {"string": "", "vector": [], "map": {}},
        "map": {"key": null},
    }))
    .unwrap();

    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("generated.rs");
    let mut uneval = uneval_static::ser::Uneval::new(File::create(path).unwrap());
    // Mappings may be required depending on your data structure
    // Serde provides type names and field names as plain identifiers;
    // these can be mapped to any output text
    uneval.add_mapping("Input".into(), "&Output".into());
    uneval.add_mapping("vector".into(), "slice".into());
    uneval.serialize(input).unwrap();
}
```

then include the generated code in `src/some_mod.rs`:

```rust
#![allow(clippy::all)]

pub struct Unit(());

pub struct Output {
    string: &'static str,
    slice: &'static [i32],
    reference: Option<&'static Output>,
    map: phf::Map<&'static str, Unit>,
}

pub static VALUE: &Output = include!(concat!(env!("OUT_DIR"), "/generated.rs"));
```

### Why?

The crate that this is forked from, [uneval](https://crates.io/crates/uneval), provides type flexibility by using trait functions such as `.into()` to convert serde types to rust types. However, this means that the output code must incur some runtime cost to initialize itself, which is suboptimal on an emotional level and potentially on a performance level as well. The code output by this crate can fit into a narrower range of types, but does not require a `lazy_static` initializer.

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
