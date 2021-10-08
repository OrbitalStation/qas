pub mod c;

/// Extra methods for class `String`
pub(crate) trait StringExt {

    /// Example:
    /// ```rust
    /// assert_eq!("(foobar)".deparentify(), "foobar")
    /// ```
    fn deparentify(&self) -> Self;
}

impl StringExt for String {
    fn deparentify(&self) -> Self {
        let mut me = self.clone();

        if me.chars().next().unwrap() == '(' && me.chars().next_back().unwrap() == ')' {
            if me[1..].chars().find(|x| *x == '(').is_some() {
                return me
            }
            me.pop();
            me.remove(0);
            me
        } else {
            me
        }
    }
}

// use proc_macro::TokenStream;
//
// #[proc_macro]
// pub fn qas(path: TokenStream) -> TokenStream {
//     let src = path.to_string();
//     assert!(src[..1].chars().next().unwrap() == '"' && src[src.len() - 1..].chars().next().unwrap() == '"');
//     let path = &src[1..src.len() - 1];
//
//
// }
