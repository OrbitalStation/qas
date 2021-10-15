///
/// `StringExt` is implemented for `String`,
/// so this macro exists to auto-implement it
/// for `str` too
///
macro_rules! auto_impl {
    ($(#[$meta:meta])* $vis:vis trait $trait:ident {
        $($(#[$metaf:meta])* fn $fn:ident(&self) -> $ret:ty;)*
    }) => {
        $(#[$meta])* $vis trait $trait {
            $($(#[$metaf])* fn $fn(&self) -> $ret;)*
        }

        impl $trait for str {
            $($(#[$metaf])*
            #[inline(always)]
            fn $fn(&self) -> $ret {
                self.to_string().$fn()
            })*
        }
    };
}

auto_impl! {
    /// Extra methods for strings
    pub(crate) trait StringExt {

        /// Example:
        ///
        /// assert_eq!("(foobar)".deparentify(), "foobar")
        ///
        fn deparentify(&self) -> String;

        /// Example:
        ///
        /// assert_eq!("foobar".parentify(), "(foobar)")
        ///
        fn parentify(&self) -> String;
    }
}

impl StringExt for String {
    fn deparentify(&self) -> String {
        let mut me = self.clone();

        if is_surrounded_with_parents(self) {
            me.pop();
            me.remove(0);
        }

        me
    }

    fn parentify(&self) -> String {
        let mut me = self.clone();

        if !is_surrounded_with_parents(self) {
            me.insert(0, '(');
            me.push(')')
        }

        me
    }
}

fn is_surrounded_with_parents(s: &String) -> bool {
    let mut stack = Vec::new();
    let mut is_surrounded = false;

    for (idx, c) in s.chars().enumerate() {
        is_surrounded = false;
        match c {
            '(' => stack.push(idx == 0),
            ')' => is_surrounded = stack.pop().unwrap_or_default(),
            _ => ()
        }
    }

    is_surrounded
}
