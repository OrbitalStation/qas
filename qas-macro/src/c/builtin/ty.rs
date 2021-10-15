use crate::StringExt;
use super::super::{FullType, TypeID};

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum BuiltinType {
    Void,

    Bool,

    SignedChar,
    UnsignedChar,

    SignedShort,
    UnsignedShort,

    SignedInt,
    UnsignedInt,

    SignedLong,
    UnsignedLong,

    Float,
    Double,

    Count
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
/// attitude signs(<, >, ==) are used not to show attitude, but dominant, i.e.
/// `a > b` means that `a` is more dominant than `b`
pub enum Dominant {
    /// a == b
    Similar,

    /// a > b
    A,

    /// a < b
    B
}

impl BuiltinType {
    pub fn add_all() {
        FullType::add_builtin("()", 0);

        FullType::add_builtin("bool", 1);

        FullType::add_builtin("i8", 1);
        FullType::add_builtin("u8", 1);

        FullType::add_builtin("i16", 2);
        FullType::add_builtin("u16", 2);

        FullType::add_builtin("i32", 4);
        FullType::add_builtin("u32", 4);

        FullType::add_builtin("i64", 8);
        FullType::add_builtin("u64", 8);

        FullType::add_builtin("f32", 4);
        FullType::add_builtin("f64", 8);
    }

    #[inline]
    fn id2self(id: &TypeID) -> Self {
        unsafe { *(&id.idx as *const usize as *const Self) }
    }

    #[inline]
    pub fn as_id(&self) -> TypeID {
        TypeID::from(*self as usize, false)
    }

    #[inline]
    pub fn is_builtin(ty: &TypeID) -> bool {
        Self::Count.as_id().idx > ty.idx || !ty.ptr.is_empty()
    }

    #[inline]
    pub fn is_floating_point(ty: &TypeID) -> bool {
        matches! { Self::id2self(ty), Self::Float | Self::Double }
    }

    pub fn is_signed(ty: &TypeID) -> bool {
        matches! { Self::id2self(ty), Self::SignedChar | Self::SignedShort | Self::SignedInt | Self::SignedLong }
    }

    pub fn is_unsigned(ty: &TypeID) -> bool {
        matches! { Self::id2self(ty), Self::UnsignedChar | Self::UnsignedShort | Self::UnsignedInt | Self::UnsignedLong }
    }

    #[inline]
    pub fn is_integer(ty: &TypeID) -> bool {
        Self::is_signed(ty) || Self::is_unsigned(ty)
    }

    #[inline]
    pub fn is_pointer(ty: &TypeID) -> bool {
        !ty.ptr.is_empty()
    }

    #[inline]
    pub fn is_arithmetic(ty: &TypeID) -> bool {
        Self::is_integer(ty) || Self::is_floating_point(ty) || Self::is_pointer(ty)
    }

    pub fn dominant(a: &TypeID, b: &TypeID) -> Dominant {
        if a == b { Dominant::Similar }
        else {
            let size_a = FullType::size(a);
            let size_b = FullType::size(b);
            
            if (size_a >= size_b) || (size_a == size_b && ((BuiltinType::is_floating_point(&a) && !BuiltinType::is_floating_point(&b))
                            || (BuiltinType::is_signed(&a) && !BuiltinType::is_signed(&b)))) { Dominant::A }
            else { Dominant::B }
        }
    }

    #[cfg(target_pointer_width = "64")]
    pub const fn usized() -> Self {
        Self::UnsignedLong
    }

    #[cfg(target_pointer_width = "32")]
    pub const fn usized() -> Self {
        Self::UnsignedInt
    }

    pub fn convert(a: &TypeID, b: &TypeID, data: &str) -> String {
        let mut data = data.to_string();

        if a.idx == BuiltinType::Count as usize || b.idx == BuiltinType::Count as usize { return data }

        let mut a = a.clone();

        if data.chars().next().unwrap() == '\"' {
            if b.ptr.is_empty() {
                panic!("cannot cast string to non-ptr type")
            }
            data.push_str(".as_ptr()");
        }

        if a == *b { return data }

        let mut convert_to_ulong = |a: &mut TypeID| {
            let ulong = BuiltinType::UnsignedLong.as_id();
            data = Self::convert(a, &ulong, &data).deparentify();
            *a = ulong;
        };

        if !a.ptr.is_empty() {
            // no handle for case `!b.ptr.is_empty()` as it is handled by ordinary cast
            if b.ptr.is_empty() {
                // if b is integer, it will be handled by ordinary cast
                if !BuiltinType::is_integer(&b) {
                    convert_to_ulong(&mut a)
                }
            }
        } else {
            // no handle for case `b.ptr.is_empty()` as it is handled by ordinary cast
            if !b.ptr.is_empty() {
                // if a is integer, it will be handled by ordinary cast
                if !BuiltinType::is_integer(&a) {
                    convert_to_ulong(&mut a)
                }
            }
        }

        let is_a_void = a.ptr.is_empty() && a.idx == BuiltinType::Void as usize;
        let is_b_void = b.ptr.is_empty() && b.idx == BuiltinType::Void as usize;
        if is_a_void && !is_b_void {
            panic!("cannot cast void to non-void")
        } else if !is_a_void && is_b_void {
            panic!("cannot cast non-void to void")
        }

        if Self::is_builtin(&a) && Self::is_builtin(&b) {
            if data.chars().next().unwrap().is_numeric() && BuiltinType::is_arithmetic(&b) {
                // has number dot or not
                match data.chars().enumerate().find(|(_, x)| *x == '.') {
                    // float
                    Some((dot, _)) => if BuiltinType::is_floating_point(&b) {
                        // float -> float, so do nothing
                        data
                    } else {
                        // float -> int, so erase dot and everything after
                        data[..dot].to_string()
                    },
                    // int
                    None => if BuiltinType::is_floating_point(&b) {
                        // int -> float, so add dot to the end
                        data + "."
                    } else {
                        // int -> int, so do nothing
                        data
                    }
                }
            } else {
                format!("({} as {})", if data.chars().find(|x| !x.is_alphanumeric()).is_some() {
                    format!("({})", data)
                } else {
                    data
                }, FullType::real(&b))
            }
        } else {
            panic!("{} is not convertible to {}", FullType::raw(&a), FullType::raw(&b))
        }
    }
}
