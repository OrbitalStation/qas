//!
//! Helper crate to provide macro `qas`
//!

mod c;
mod string_ext;

pub(crate) use string_ext::*;

use proc_macro::TokenStream;

#[proc_macro]
pub fn qas(path: TokenStream) -> TokenStream {
    let path = path.to_string();
    let path = path.trim();

    assert!(path[..1].chars().next().unwrap() == '"' && path[path.len() - 1..].chars().next().unwrap() == '"', "input should be a string");
    let path = &path[1..path.len() - 1];

    let code = read_file(&path);

    let c = c::start(path, code);
    println!("{}", c);
    c.parse().unwrap()
}

pub(crate) fn read_file(file: &str) -> String {
    match std::fs::read_to_string(file) {
        Ok(x) => x,
        Err(e) => panic!("failed to find \"{}\":\n\t{:?}", file, e)
    }
}
