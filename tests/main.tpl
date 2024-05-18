use serde::Serialize;
use std::fs::File;

{definition}

fn main() {{
    let mut uneval = uneval_static::ser::Uneval::new(File::create("{path}/generated.rs").unwrap());
    {name_mappings}
    uneval.serialize({value}).unwrap();
}}
