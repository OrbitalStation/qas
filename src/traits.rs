///
/// Helper trait to convert C string into Rust one.
///
/// Example:
///
/// ```rust
/// use qas::prelude::*;
///
/// qas!("tests/c/string.c");
///
/// fn main() {
///     assert_eq!(unsafe { hi().to_rust() }, "Hi there")
/// }
/// ```
///
pub unsafe trait CStringToRust {
    ///
    /// Unsafe because caller has to guarantee that `self` is a valid pointer
    ///
    unsafe fn to_rust(&self) -> &str;
}

///
/// Helper trait to convert Rust string into C one.
///
/// WARNING: the `to_c` method <i>does</i> check if string has null-terminator,
///
/// but `to_c_unchecked` <i>doesn't</i>.
///
/// if you want to convert a <i>C string</i> that was converted into Rust, you can just call
///
/// `to_c` or even `to_c_unchecked`,
///
/// but if you want to convert a <i>native</i> Rust string into C String,
///
/// you <i>really</i> have to ensure that you string contains null-terminator - that's what `to_c` does.
///
/// Example:
///
/// ```rust
/// use qas::prelude::*;
///
/// qas!("tests/c/string.c");
///
/// fn main() {
///     // contains null-terminator!
///     assert_eq!(unsafe { return_same("Rusty\0".to_c()).to_rust() }, "Rusty");
///
///     assert_eq!(unsafe { hi().to_rust().to_c().to_rust() }, "Hi there")
/// }
/// ```
///
pub unsafe trait RustStringToC {
    ///
    /// Unsafe because caller has to guarantee that `self` contains null-terminator
    ///
    unsafe fn to_c(&self) -> *const u8;

    ///
    /// Unsafe because C string is returned may not be valid
    ///
    unsafe fn to_c_unchecked(&self) -> *const u8;
}

///
/// Helper trait to implement things such as `++` operator
///
pub trait Integer: Copy + Sized {
    /// Add x to self
    ///
    /// Example:
    /// ```rust
    /// use qas::prelude::*;
    ///
    /// let mut x: i32 = 18;
    /// x.add_one_u8(1);
    /// assert_eq!(x, 19)
    /// ```
    fn add_one_u8(&mut self, x: u8);

    /// Subtract x to self
    ///
    /// Example:
    /// ```rust
    /// use qas::prelude::*;
    ///
    /// let mut x: i32 = 18;
    /// x.sub_one_u8(1);
    /// assert_eq!(x, 17)
    /// ```
    fn sub_one_u8(&mut self, x: u8);
}

unsafe impl CStringToRust for *const u8 {
    unsafe fn to_rust(&self) -> &str {
        let len = strlen(*self);
        let raw = core::slice::from_raw_parts(*self, len);

        core::str::from_utf8(raw).unwrap()
    }
}

unsafe impl RustStringToC for str {
    unsafe fn to_c(&self) -> *const u8 {
        match self.chars().next_back() {
            Some(zero) if zero == '\0' => (),
            _ => {
                let len = self.len();
                let start = self.as_ptr();
                let byte_after_address = (start as usize + len) as *const u8;
                let byte_after = *byte_after_address;
                assert_eq!(byte_after, b'\0', "string is not terminated by null")
            }
        }
        self.to_c_unchecked()
    }

    #[inline(always)]
    unsafe fn to_c_unchecked(&self) -> *const u8 {
        self.as_ptr()
    }
}

///
/// Calculates length of C String
///
unsafe fn strlen(mut s: *const u8) -> usize {
    let start = s;
    while *s != 0 {
        s = (s as usize + 1) as *const u8;
    }
    s as usize - start as usize
}

/// Helper macro to auto-implement trait `Integer` for all integers(and not only)
macro_rules! impl_num {
    ($($ty:ident)*) => {
        $(impl Integer for $ty {
            #[inline(always)]
            fn add_one_u8(&mut self, x: u8) {
                *self += x as Self;
            }

            #[inline(always)]
            fn sub_one_u8(&mut self, x: u8) {
                *self -= x as Self;
            }
        })*
    };
}

impl_num!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize);
