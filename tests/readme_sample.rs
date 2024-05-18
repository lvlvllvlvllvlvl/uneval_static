use serde::Deserialize;
use serde_json::json;
use std::{collections::HashMap, fs::File};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct Unit(());

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct Input {
    string: String,
    vector: Vec<i32>,
    reference: Option<Box<Input>>,
    map: HashMap<String, Unit>,
}

#[test]
fn readme_example() {
    let input = Input::deserialize(json!({
        "string": "test",
        "vector": [1,2,3],
        "reference": {"string": "", "vector": [], "map": {}},
        "map": {"key": null},
    }))
    .unwrap();

    //let path = Path::new(&env::var("OUT_DIR").unwrap()).join("generated.rs");
    let mut uneval =
        uneval_static::ser::Uneval::new(File::create("test_fixtures/readme_generated.rs").unwrap());
    // Mappings may be required depending on your data structure
    // Serde provides type names as plain identifiers; these can be mapped to any output text
    uneval.add_mapping("Input".into(), "&Output".into());
    uneval.add_mapping("vector".into(), "slice".into());
    uneval.serialize(input).unwrap();

    std::fs::write(
        "test_fixtures/readme.rs",
        r#"
#![allow(clippy::all)]

pub struct Unit(());

pub struct Output {
    string: &'static str,
    slice: &'static [i32],
    reference: Option<&'static Output>,
    map: phf::Map<&'static str, Unit>,
}

//include!(concat!(env!("OUT_DIR"), "/generated.rs"))
static VALUE: &Output = include!("readme_generated.rs");
    
fn main() {
    assert_eq!(VALUE.string, "test");
}
    "#,
    )
    .unwrap();
    let trybuild = trybuild::TestCases::new();
    trybuild.pass("test_fixtures/readme.rs");
}
