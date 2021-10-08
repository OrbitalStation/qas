use super::super::{Type, TypeID};

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
        Type::add_builtin("()",  0);

        Type::add_builtin("bool", 1);

        Type::add_builtin("i8",  1);
        Type::add_builtin("u8",  1);

        Type::add_builtin("i16", 2);
        Type::add_builtin("u16", 2);

        Type::add_builtin("i32", 4);
        Type::add_builtin("u32", 4);

        Type::add_builtin("i64", 8);
        Type::add_builtin("u64", 8);

        Type::add_builtin("f32", 4);
        Type::add_builtin("f64", 8);
    }

    #[inline]
    fn id2self(id: TypeID) -> Self {
        unsafe { *(&id as *const TypeID as *const Self) }
    }

    #[inline]
    pub const fn as_id(&self) -> TypeID {
        *self as TypeID
    }

    #[inline]
    pub const fn is_builtin(ty: TypeID) -> bool {
        Self::Count.as_id() > ty
    }

    #[inline]
    pub fn is_floating_point(ty: TypeID) -> bool {
        matches! { Self::id2self(ty), Self::Float | Self::Double }
    }

    pub fn is_signed(ty: TypeID) -> bool {
        matches! { Self::id2self(ty), Self::SignedChar | Self::SignedShort | Self::SignedInt | Self::SignedLong }
    }

    pub fn is_unsigned(ty: TypeID) -> bool {
        matches! { Self::id2self(ty), Self::UnsignedChar | Self::UnsignedShort | Self::UnsignedInt | Self::UnsignedLong }
    }

    #[inline]
    pub fn is_integer(ty: TypeID) -> bool {
        Self::is_signed(ty) || Self::is_unsigned(ty)
    }

    #[inline]
    pub fn is_arithmetic(ty: TypeID) -> bool {
        Self::is_integer(ty) || Self::is_floating_point(ty)
    }

    pub fn dominant(a: TypeID, b: TypeID) -> Dominant {
        if a == b { Dominant::Similar }
        else {
            let size_a = Type::size(a);
            let size_b = Type::size(b);
            
            if (size_a > size_b) || (size_a == size_b && ((BuiltinType::is_floating_point(size_a) && !BuiltinType::is_floating_point(size_b))
                            || (BuiltinType::is_signed(size_a) && !BuiltinType::is_signed(size_b)))) { Dominant::A }
            else { Dominant::B }
        }
    }

    pub fn convert(a: TypeID, b: TypeID, data: &str) -> String {
        let data = data.to_string();

        if a == b { data }
        else if Self::is_builtin(a) && Self::is_builtin(b) {
            let has_space = data.chars().find(|x| x.is_whitespace()).is_some();

            // If there's no alphabetic it is a number
            if !has_space && data.chars().find(|x| x.is_alphabetic()).is_none() && BuiltinType::is_arithmetic(b) {
                // has number dot or not
                match data.chars().enumerate().find(|(_, x)| *x == '.') {
                    // float
                    Some((dot, _)) => if BuiltinType::is_floating_point(b) {
                        // float -> float, so do nothing
                        data
                    } else {
                        // float -> int, so erase dot and everything after
                        data[..dot].to_string()
                    },
                    // int
                    None => if BuiltinType::is_floating_point(b) {
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
                }, Type::real(b))
            }
        } else {
            panic!("{} is not convertible to {}", Type::raw(a), Type::raw(b))
        }
    }
}
