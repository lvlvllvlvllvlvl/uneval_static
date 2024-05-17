use regex::Regex;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fmt::Write as OtherWrite,
    fs::{create_dir, read_to_string, File},
    io::Write,
    path::PathBuf,
    str::FromStr,
};
use toml::from_str;

#[derive(Deserialize, Default)]
struct Data {
    main_type: String,
    definition: String,
    value: String,
    test_values: Option<HashMap<String, String>>,
}

impl Data {
    fn write(&self, name: &str) {
        let path = PathBuf::from_str(format!("test_fixtures/{name}/").as_str()).unwrap();
        if !path.exists() {
            create_dir(&path).unwrap();
        }
        let abs = path
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
            .replace('\\', "/");
        write!(
            File::create(format!("test_fixtures/{name}/main.rs")).unwrap(),
            include_str!("main.tpl"),
            value = self.value,
            path = abs,
            definition = self.definition
        )
        .unwrap();
        let definition = Regex::new("Vec<([^>]*)>")
            .unwrap()
            .replace_all(self.definition.as_str(), "&'static [$1]");
        let definition = definition
            .replace("String", "&'static str")
            .replace(", Serialize", "")
            .replace("std::collections::HashMap", "phf::Map");
        write!(
            File::create(format!("test_fixtures/{name}/user.rs")).unwrap(),
            include_str!("user.tpl"),
            ser_type = self.main_type,
            assert = match &self.test_values {
                Some(map) => map.iter().fold(String::new(), |mut s, (k, v)| {
                    writeln!(s, "assert_eq!(ITEM.{k}, {v});").unwrap();
                    s
                }),
                None => format!("assert_eq!(ITEM, {});", self.value.replace("vec!", "&")),
            },
            definition = definition
        )
        .unwrap();
    }
}

#[test]
fn main() {
    let toml = read_to_string("test_fixtures/data.toml").unwrap();
    let data: HashMap<String, Data> = from_str(&toml).unwrap();
    let b = trybuild::TestCases::new();
    data.into_iter().for_each(|(name, value)| {
        value.write(&name);
        b.pass(format!("test_fixtures/{name}/main.rs"));
        b.pass(format!("test_fixtures/{name}/user.rs"));
    });
}
