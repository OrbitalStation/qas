pub struct Macro {
    pub name: String,
    pub value: String
}

pub struct SpecialMacro {
    pub name: &'static str,
    pub value: fn() -> String
}

impl Macro {
    pub fn predefine_all() {
        Self::add("__STDC__", "1");
        Self::add("__STDC_VERSION__", "199901");

        Self::add("__RUST__", "");

        Self::add("__qas_minor__", "0");
        Self::add("__qas_major__", "0");
        Self::add("__qas_patch__", "0");

        #[cfg(target_os = "linux")]
        Self::add("__linux__", "");

        // special

        Self::add_special("__func__", || format!("\"{}\"", super::Function::get().name))
    }

    #[inline]
    pub fn macros() -> &'static mut Vec <Macro> {
        static mut MACROS: Vec <Macro> = Vec::new();
        unsafe { &mut MACROS }
    }

    #[inline]
    pub fn special_macros() -> &'static mut Vec <SpecialMacro> {
        static mut MACROS: Vec <SpecialMacro> = Vec::new();
        unsafe { &mut MACROS }
    }

    pub fn is_defined(name: &str) -> bool {
        Self::find(name).is_some()
    }

    pub fn add <S: ToString> (name: S, value: S) {
        fn _add(name: String, value: String) {
            match Macro::macros().iter_mut().find(|x| x.name == name) {
                Some(m) => m.value = value,
                None => if Macro::special_macros().iter().find(|x| x.name == name).is_none() {
                    Macro::macros().push(Macro { name, value })
                }
            }
        }

        _add(name.to_string(), value.to_string())
    }

    pub fn add_special(name: &'static str, value: fn() -> String) {
        Self::special_macros().push(SpecialMacro { name, value })
    }

    pub fn find(name: &str) -> Option <String> {
        Self::macros().iter().find(|x| x.name == name).map(|x| x.value.clone())
    }
}
