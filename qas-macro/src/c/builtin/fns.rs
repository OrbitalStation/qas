#![allow(dead_code, unused_imports)]

use super::super::{TypeID, BuiltinType};

#[derive(Clone)]
pub struct BuiltinFunction {
    pub name: &'static str,
    pub real: String,
    pub args: Vec <&'static TypeID>,
    pub ret: &'static TypeID
}

impl BuiltinFunction {
    pub fn add_all() {
        // Self::add("__sinf", "sinf", vec![&TypeID {
        //     idx: BuiltinType::SignedInt as usize,
        //     ptr: 0,
        //     mutable: true
        // }], &TypeID {
        //     idx: BuiltinType::SignedInt as usize,
        //     ptr: 0,
        //     mutable: true
        // })
    }

    pub fn add(name: &'static str, real: &'static str, args: Vec <&'static TypeID>, ret: &'static TypeID) {
        Self::fns().push(Self {
            name,
            real: format!("::qas::builtin::{}", real),
            args,
            ret
        })
    }

    pub fn fns() -> &'static mut Vec <BuiltinFunction> {
        static mut FNS: Vec <BuiltinFunction> = Vec::new();
        unsafe { &mut FNS }
    }
}
