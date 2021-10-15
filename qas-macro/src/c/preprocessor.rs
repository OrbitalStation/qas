use chrono::{Datelike, Timelike};

pub struct Macro {
    pub name: String,
    pub value: String
}

impl Macro {
    pub fn predefine_all(file: &str) {
        Self::add("__STDC__", "1");
        Self::add("__STDC_VERSION__", "199901");

        Self::add("__RUST__", "");

        Self::add("__qas_minor__", "0");
        Self::add("__qas_major__", "0");
        Self::add("__qas_patch__", "0");

        #[cfg(target_os = "linux")]
        Self::add("__linux__", "");

        unsafe {
            FILE = format!("\"{}\"", std::fs::canonicalize(file).unwrap().as_path().to_str().unwrap());
            LINE = 1
        }

        #[cfg(not(debug_assertions))]
        Self::add("NDEBUG", "");

        let now = chrono::Local::now();
        Self::add("__DATE__", format!("\"{} {:02} {}\"", match now.month0() {
             0 => "Jan",
             1 => "Feb",
             2 => "Mar",
             3 => "Apr",
             4 => "May",
             5 => "Jun",
             6 => "Jul",
             7 => "Aug",
             8 => "Sep",
             9 => "Oct",
            10 => "Nov",
            11 => "Dec",
            _ => unreachable!()
        }, now.day(), now.year()));
        Self::add("__TIME__", format!("\"{:02}:{:02}:{:02}\"", now.hour(), now.minute(), now.second()));
    }

    #[inline]
    pub fn macros() -> &'static mut Vec <Macro> {
        static mut MACROS: Vec <Macro> = Vec::new();
        unsafe { &mut MACROS }
    }

    pub fn is_defined(name: &str) -> bool {
        Self::find(name).is_some()
    }

    pub fn add <S1: ToString, S2: ToString> (name: S1, value: S2) {
        fn _add(name: String, value: String) {
            match Macro::macros().iter_mut().find(|x| x.name == name) {
                Some(m) => m.value = value,
                None => Macro::macros().push(Macro { name, value })
            }
        }

        _add(name.to_string(), value.to_string())
    }

    pub fn find(name: &str) -> Option <String> {
        Self::macros().iter().find(|x| x.name == name).map(|x| x.value.clone())
    }
}

pub static mut LINE: usize = 1;
pub static mut FILE: String = String::new();
