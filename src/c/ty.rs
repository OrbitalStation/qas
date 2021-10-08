use super::BuiltinType;

pub type TypeID = usize;

pub struct Type {
    pub raw:  String,
    pub real: String,
    pub size: usize
}

impl Type {
    #[inline]
    pub fn types() -> &'static mut Vec <Type> {
        static mut TYPES: Vec <Type> = Vec::new();
        unsafe { &mut TYPES }
    }

    pub fn add_builtin(ty: &str, size: usize) {
        Self::types().push(Type { raw: String::new(), real: ty.to_string(), size })
    }

    #[inline]
    pub fn real(ty: TypeID) -> &'static String {
        &Self::types()[ty].real
    }

    #[inline]
    pub fn raw(ty: TypeID) -> &'static String {
        if BuiltinType::is_builtin(ty) {
            &Self::types()[ty].real
        } else {
            &Self::types()[ty].raw
        }
    }

    #[inline]
    pub fn size(ty: TypeID) -> usize {
        Self::types()[ty].size
    }
}
