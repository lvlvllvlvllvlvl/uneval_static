use serde::Serialize;
use uneval_static::to_file;

{definition}

fn main() {{
    to_file({value}, "{path}/generated.rs").unwrap();
}}
