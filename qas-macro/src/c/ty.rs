use super::{BuiltinType, Function};

#[derive(Debug, Clone, Eq)]
pub struct TypeID {
    pub idx: usize,
    pub ptr: bit_vec::BitVec <u8>,
    pub mutable: bool
}

impl PartialEq for TypeID {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx && self.ptr == other.ptr
    }
}

impl TypeID {
    pub fn from(idx: usize, mutable: bool) -> Self {
        Self {
            idx,
            ptr: Default::default(),
            mutable
        }
    }

    pub fn mutable(&self) -> Self {
        Self {
            mutable: true,
            ..self.clone()
        }
    }
}

pub struct FullType {
    pub raw:  String,
    pub real: String,
    pub size: usize
}

impl FullType {
    #[inline]
    pub fn types() -> &'static mut Vec <FullType> {
        static mut TYPES: Vec <FullType> = Vec::new();
        unsafe { &mut TYPES }
    }

    pub fn add_builtin(ty: &str, size: usize) {
        Self::types().push(Self { raw: String::new(), real: ty.to_string(), size })
    }

    pub fn real(ty: &TypeID) -> String {
        format!("{}{}", if ty.ptr.is_empty() {
            String::new()
        } else {
            let mut s = String::new();
            let mut i = 0;
            while i < ty.ptr.len() {
                s.push('*');
                if ty.ptr[i] {
                    s.push_str("mut")
                } else {
                    s.push_str("const")
                }
                s.push(' ');
                i += 1
            }
            s
        }, &Self::types()[ty.idx].real)
    }

    pub fn raw(ty: &TypeID) -> String {
        format!("{}{}",
            if ty.mutable { "" } else { "const " },
            if BuiltinType::is_builtin(ty) {
                &Self::types()[ty.idx].real
            } else {
                &Self::types()[ty.idx].raw
            }
        )
    }

    pub fn size(ty: &TypeID) -> usize {
        Self::types()[ty.idx].size
    }
}

pub struct AliasType {
    pub name: String,
    pub id: TypeID,
    pub is_in_function: bool
}

impl AliasType {
    #[inline]
    pub fn types() -> &'static mut Vec <AliasType> {
        static mut TYPES: Vec <AliasType> = Vec::new();
        unsafe { &mut TYPES }
    }

    pub fn add(name: String, id: TypeID) {
        Self::types().push(Self {
            name,
            id,
            is_in_function: Function::fns().len() != 0
        })
    }

    pub fn find(name: &str) -> Option <&'static TypeID> {
        Self::types().iter().find(|x| x.name == name).map(|x| &x.id)
    }

    pub fn clear() {
        let mut i = 0;
        while i < Self::types().len() {
            if Self::types()[i].is_in_function {
                Self::types().remove(i);
            } else {
                i += 1
            }
        }
    }
}
