use super::ty::TypeID;

pub struct Function {
    pub name: String,
    pub ret: TypeID,
    pub args: usize,
    pub lets: Vec <Let>,
    pub attrs: Vec <String>
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
    pub fn pop() {
        Self::fns().pop();
    }

    #[inline]
    pub fn get() -> &'static mut Function {
        Self::fns().last_mut().unwrap()
    }

    pub fn type_of_let(&self, name: &str) -> Option <TypeID> {
        self.lets.iter().find(|x| x.name == name).map(|x| x.ty)
    }

    pub fn tabs() -> String {
        "\t".repeat(Self::fns().len())
    }
}

#[derive(Debug)]
pub struct Let {
    pub name: String,
    pub ty: TypeID
}
