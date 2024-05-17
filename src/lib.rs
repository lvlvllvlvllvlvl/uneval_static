//! Serde serializer generating Rust code.
//!
//! This crate can be used to "embed" something into code, having only some serialized
//! data, like JSON or YAML. This way, you'll mostly escape runtime cost of deserialization,
//! nearly as if you've written the same data directly in code by hand.
//! Of course, in most cases this cost is already negligible, but for crates which use
//! large blobs of data this crate can come in handy, improving startup times, and can
//! eliminate the need for `serde` as runtime dependency.
//!
//! ## Usage
//! In general, to embed some code into crate, you have to use the build script
//! and [`include!`][include] macro. Inside the build script, you'll generate
//! some code with one of the [functions][funcs] provided by `uneval`,
//! and then include the generated file, like this:
//! ```ignore
//! static VALUE: Type = include!(concat!(env!(OUT_DIR), "/file_name.rs"));
//! ```
//!
//! ## How does it work?
//!
//! Of course, we can't always directly construct the code for the desired value (more on this
//! in the [Limitations](#limitations) section below).
//! However, in many cases the information provided by Serde is enough.
//!
//! For every case, we'll provide an example of how the generated code can look like, as a sequence of
//! assignment statements, representing both the definition of your target struct field and the literal
//! value that will be assigned to it.
//!
//! ### Primitives
//!
//! Number literals, such as `i8` or `f32`, are directly written into the output. The only tricky part
//! is that we have to use suffixed literals, e.g. `1u8` or `1.1f64` - otherwise we'd run into the problem
//! with the float values which are in fact integers, since they would be output as integer literals,
//! not as float ones (i.e. `1` and not `1.0`) and so wouldn't typecheck.
//!
//! Boolean and character literals are also simply written directly - no surprises here.
//!
//! Example:
//! ```
//! let _: i8 = 12i8;
//! let _: u128 = 12345u128;
//! let _: f32 = -1f32;
//! let _: f64 = 12345.6789f64;
//! let _: char = 'c';
//! let _: bool = true;
//! ```
//!
//! ### Strings
//! Strings are output as string literals; your struct field should be of type `&'static str` to hold it.
//!
//! Example:
//! ```
//! let _: &'static str = "string value";
//! ```
//!
//! Byte strings are handled as byte sequences, [as recommended by Serde itself][::serde::Serializer::serialize_bytes],
//! and so we'll discuss them [below](#vec-like-types-sequences).
//!
//! ### Tuple structs and unit values
//!
//! Unit type (`()`), unit structs and unit variants (including `None`) are emitted simply by using
//! the type name. Tuple structs and variants (and newtype-flavored ones, including `Some`)
//! are emitted by writing  their name (with the enum name, if necessary), parenthesis,
//! and serializing the inner values.
//!
//! Example:
//! ```
//! struct TupleStruct((), Option<u8>, Option<u8>);
//! let _: TupleStruct = TupleStruct((), None, Some(1u8));
//! ```
//!
//! ### Vec-like types (sequences)
//!
//! If the input data contains a variably-typed sequence such as Vec<T>, the target struct should have
//! a field of type `&'static [T]`.
//!
//! Example:
//! ```
//! let _: &'static [u32] = &[1u32, 2u32, 3u32];
//! ```
//!
//! ### Tuples and arrays
//!
//! The problem is that Serde doesn't distinguish between this two kinds of values: they both are treated
//! as sequences with known length, called "tuples" internally. Thus you will need to declare any fixed-length
//! sequence fields in your target struct as tuples, even if all elements are the same type. If necessary,
//! a feature flag could be added to convert them all to arrays instead.
//! ```
//! struct SourceStruct([u32; 3]);
//! struct TargetStruct((u32, u32, u32));
//! let _ = TargetStruct((1u32, 2u32, 3u32));
//! ```
//!
//! #### Zero-sized arrays
//!
//! Likewise, arrays of length zero are output as the unit type.
//! ```
//! struct SourceStruct([u32; 0]);
//! struct TargetStruct(());
//! let _ = TargetStruct(());
//! ```
//!
//! ### Maps
//!
//! Maps are generated with [phf_codegen]. If your data includes a map you will need to add [phf] to your dependencies.
//! Currently only string keys are implemented; other primitive types could be added, but that would require a refactor.
//!
//! Example:
//! ```
//! # type T = ();
//! # let phf_internals = phf::Map::new();
//! let _: phf::Map<&'static str, T> = ::phf::Map { ..phf_internals };
//! ```
//!
//! ### Structs
//!
//! Last but not the least, this case is relatively simple. Emitted code is simply the struct construction -
//! i.e. the struct name, the curly braces and a list of pairs of the form `{field name}: {serialized value}`.
//!
//! Example:
//! ```
//! struct Struct { boolean: bool, number: i32, string: &'static str }
//! let _: Struct = Struct {
//!     boolean: true,
//!     number: 1i32,
//!     string: "string"
//! };
//! ```
//!
//! ## Limitations
//! There are some cases when `uneval` will be unable to generate valid code. Namely:
//! 1. Since Serde doesn't provide us the full path to the type in question (and in most cases it's simply unable to),
//! all the structs and enums used during value construction must be in scope.
//! As a consequence, all of them must have distinct names - otherwise, there will be name clashes.
//! 2. This serializer is intended for use with derived implementation. It may return bogus results
//! when used with customized `Serialize`.
//!
//! [include]: https://doc.rust-lang.org/stable/std/macro.include.html

pub mod error;
pub mod funcs;
pub mod ser;

pub use funcs::{to_file, to_out_dir, to_string, write};
