use super::{TypeID, BuiltinFunction};

bitflags::bitflags! {
    pub struct FnFlags: u8 {
        const SAFE   = 0b01;
        const PUBLIC = 0b10;
    }
}

pub struct Function {
    pub name: String,
    pub ret: TypeID,
    pub args: usize,
    pub lets: Vec <Let>,
    pub attrs: Vec <String>,
    pub flags: FnFlags
}

impl Function {
    pub fn fns() -> &'static mut Vec <Function> {
        static mut FNS: Vec <Function> = Vec::new();
        unsafe { &mut FNS }
    }

    #[inline]
    pub fn add(f: Function) {
        Self::fns().push(f)
    }

    #[inline]
    pub fn get() -> &'static mut Function {
        Self::fns().last_mut().unwrap()
    }

    pub fn type_of_let(&self, name: &str) -> Option <&TypeID> {
        self.lets.iter().find(|x| x.name == name).map(|x| &x.ty)
    }

    pub fn should_be_safe(&self) -> bool {
        self.attrs.iter().find(|x| *x == "%S").is_some()
    }

    pub fn as_builtin(&'static self) -> BuiltinFunction {
        BuiltinFunction {
            name: "",
            real: self.name.to_string(),
            args: self.lets[..self.args].iter().map(|x| &x.ty).collect(),
            ret: &self.ret
        }
    }

    pub fn check_and_make_mutable_on_require(name: &str) {
        match Self::get().lets.iter_mut().find(|x| x.name == name) {
            Some(x) => x.mutable = true,
            None => ()
        }
    }
}

#[derive(Debug)]
pub struct Let {
    pub name: String,
    pub mutable: bool,
    pub ty: TypeID
}

pub struct Tab;

impl Tab {
    pub fn number_tabs() -> &'static mut usize {
        static mut TAB: usize = 0;
        unsafe { &mut TAB }
    }

    pub fn tabs_num() -> usize {
        *Self::number_tabs() + Function::fns().len()
    }

    pub fn tabs() -> String {
        "\t".repeat(Self::tabs_num())
    }

    pub fn tabs_nl() -> String {
        String::from("\n") + &Self::tabs()
    }
}
