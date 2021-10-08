mod builtin;
mod fns;
mod op;
mod preprocessor;
mod ty;
mod comment;

use crate::StringExt;
use builtin::*;
use fns::*;
use op::*;
use ty::*;
use preprocessor::*;
use comment::*;

#[derive(Debug)]
pub struct Expr {
    pub name: String,
    pub ty: TypeID
}

impl Expr {
    #[inline]
    pub const fn new(name: String, ty: TypeID) -> Self {
        Self { name, ty }
    }

    pub fn convert(&mut self, to: TypeID) {
        self.name = BuiltinType::convert(self.ty, to, &self.name);
        self.ty = to
    }
}

fn boolify(value: evalexpr::Value) -> bool {
    use evalexpr::Value;

    match value {
        Value::String(x) => x.len() != 0,
        Value::Float(x) => x != 0.0,
        Value::Int(x) => x != 0,
        Value::Boolean(x) => x,
        Value::Empty => false,
        _ => unimplemented!()
    }
}

peg::parser! {
    grammar clang() for str {
        rule digit() -> &'input str
            = x:$(['0'..='9'])

        rule letter() -> &'input str
            = x:$(['a'..='z' | 'A'..='Z' | '_' | '$'])

        rule name() -> &'input str
            = x:$(letter() (letter() / digit())*)

        rule __num_suffix() -> &'static str
            = $("l" / "L" / "ll" / "LL") { "i64" }
            / $("u" / "U") { "u64" }

        rule num() -> String
            = num:$("-"? digit()+ "."? digit()*) suffix:__num_suffix()? { num.to_string() + suffix.unwrap_or_default() }

        rule punct() -> &'input str
            = x:$(['+' | '-' | '*' | '/' | '%' | '&' | '|' | '=' | '<' | '>' | '!' | '^' | '~']*)

        rule __string_one() -> String = precedence! {
            "\\\"" { String::from("\"") }
            x:$([^ '\"']) { x.to_string() }
        }

        rule string() -> String
            = "\"" x:__string_one()* "\"" { x.join("") }

        rule binop() -> Binop = op:punct() {?
            for i in 0..(Binop::Count as u8) {
                let i = unsafe { *(&i as *const u8 as *const Binop) };
                if i.as_str() == op { return Ok(i) }
            }
            Err("is not a binary operator")
        }

        rule oneop() -> Oneop = op:punct() {?
            for i in 0..(Oneop::Count as u8) {
                let i = unsafe { *(&i as *const u8 as *const Oneop) };
                if i.as_str() == op { return Ok(i) }
            }
            Err("is not a unary operator")
        }

        rule _ = [' ' | '\n' | '\t']*

        rule __ = [' ' | '\n' | '\t']+

        rule ___ = [' ' | '\t']*

        rule ____() = [' ' | '\t']+

        rule __builtin_try_ty() -> BuiltinType
            = "void" { BuiltinType::Void }

            / "_Bool" { BuiltinType::Bool }

            / "signed" _ "char" { BuiltinType::SignedChar }
            / ("unsigned" _ "char" / "char") { BuiltinType::UnsignedChar }

            / ("signed" _ "short" _ "int" / "signed" _ "short" / "short" _ "int" / "short") { BuiltinType::SignedShort }
            / ("unsigned" _ "short" _ "int" / "unsigned" _ "short") { BuiltinType::UnsignedShort }

            / ("signed" _ "long" _ "long" _ "int" / "long" _ "long" _ "int" / "signed" _ "long" _ "long" / "signed" _ "long" _ "int" / "signed" _ "long" / "long" _ "int" / "long" _ "long" / "long") { BuiltinType::SignedLong }
            / ("unsigned" _ "long" _ "long" _ "int" / "unsigned" _ "long" _ "int" / "unsigned" _ "long" _ "long" / "unsigned" _ "long") { BuiltinType::UnsignedLong }

            / "float" { BuiltinType::Float }
            / "double" { BuiltinType::Double }

            / ("signed" _ "int" / "int" / "signed") { BuiltinType::SignedInt }
            / ("unsigned" _ "int" / "unsigned") { BuiltinType::UnsignedInt }

        pub(in self) rule try_ty() -> Option <TypeID> = precedence! {
            x:__builtin_try_ty() { Some(x.as_id()) }

            x:name() {
                let mut i = 0;
                while i < Type::types().len() {
                    if Type::real(i) == x { return Some(i) }
                    i += 1
                }
                None
            }
        }

        pub(in self) rule ty() -> TypeID = ty:try_ty() {?
            match ty {
                Some(x) => Ok(x),
                None => Err("expected type")
            }
        }

        rule var() -> &'input str = x:name() {?
            match try_ty(&x) {
                Ok(id) => match id {
                    Some(_) => Err(""),
                    None => Ok(x)
                },
                Err(_) => Err("")
            }
        }

        rule expr_ty(ty: bool) -> Expr = precedence! {
            "(" _ x:expr_ty(ty) _ ")" { Expr::new(format!("({})", x.name), x.ty) }

            cond:(@) _ "?" _ s1:expr_ty(ty) _ ":" _ s2:@ {
                if s1.ty != s2.ty {
                    panic!("branches of ?: cannot have different types")
                }
                let cond = BuiltinType::convert(cond.ty, BuiltinType::Bool.as_id(), &cond.name).deparentify();
                Expr::new(format!("if {} {{\n{tabs}\t{}\n{tabs}}} else {{\n\t{tabs}{}\n{tabs}}}", cond, s1.name, s2.name, tabs = Function::tabs()), s1.ty)
            }

            oneop:oneop() _ x:(@) { oneop.operate(x) }

            x:(@) _ binop:binop() _ y:@ { binop.operate(x, y) }

            i:num() { Expr::new(i.to_string(), if i.chars().find(|x| *x == '.').is_some() { BuiltinType::Float.as_id() } else { BuiltinType::SignedInt.as_id() }) }

            i:var() { Expr::new(i.to_string(), Function::get().type_of_let(&i).expect("unknown variable")) }
        }

        rule expr() -> Expr = expr_ty(false)

        rule __stmt_add(name: &str, ret: TypeID, ty0: Option <TypeID>, arg0: Option <&str>, args: Vec <Let>) = _ {
            let mut args = args;
            match ty0 {
                Some(ty0) => match arg0 {
                    Some(arg0) => {
                        args.insert(0, Let {
                            name: arg0.to_string(),
                            ty: ty0
                        })
                    },
                    None => panic!("cannot have anonymous parameters")
                },
                None => match arg0 {
                    Some(x) => panic!("{} does not have type", x),
                    None => ()
                }
            }

            Function::add(Function {
                name: name.to_string(),
                ret,
                args: args.len(),
                lets: args,
                attrs: Vec::new()
            })
        }

        rule __stmt_arg() -> Let = "," _ ty:ty() _ name:var() _ {
            Let { name: name.to_string(), ty }
        }

        rule __stmt_fn_attr_inside() -> String
            = "always_inline" { String::from("inline(always)") }
            / "never_inline"  { String::from("inline(never)") }
            / "noreturn"      { String::from("%N") }

        rule __stmt_fn_attr() -> String
            = "inline" _ { String::from("inline") }
            / "__ATTR__" _ "((" _ attr:__stmt_fn_attr_inside() _ "))" _ { attr }

        rule __stmt_fn_attrs() -> Vec <String> = attrs:__stmt_fn_attr()*

        rule __stmt_return_is_last() -> () = &"}"

        rule __stmt_return_possible_no() -> Expr = __ e:expr() { e }

        rule stmt() -> String = precedence! {
            "return" e:__stmt_return_possible_no()? _ ";" _ is_last:__stmt_return_is_last()? _ {
                if e.is_none() {
                    if Function::get().ret != BuiltinType::Void.as_id() {
                        panic!("expected value")
                    }
                    return String::new()
                }
                let e = e.unwrap();
                let result = BuiltinType::convert(e.ty, Function::get().ret, &e.name).deparentify();
                if is_last.is_some() {
                    format!("{}", result)
                } else {
                    format!("return {};", result)
                }
            }

            attrs:__stmt_fn_attrs() ret:ty() __ name:var() _ "(" _ ty0:ty()? __ arg0:var()? _ args:__stmt_arg()* ","? ")" __stmt_add(name, ret, ty0, arg0, args) "{" _ body:clang_case("\n\t") _ "}" _ {
                if (1..attrs.len()).any(|i| attrs[i..].contains(&attrs[i - 1])) {
                    panic!("cannot have duplicate attributes")
                }
                Function::get().attrs = attrs;
                let x = format!("\n{}fn {}({}){} {{\n\t{}\n}}\n",
                    {
                        let mut s = String::new();
                        for attr in &Function::get().attrs {
                            if attr.chars().next().unwrap() != '%' {
                                s.push_str(format!("#[{}]\n", attr).as_str())
                            }
                        }
                        s
                    },
                    Function::get().name,
                    {
                        let mut s = String::new();
                        for arg in Function::get().lets[..Function::get().args].iter() {
                            s.push_str(format!("{}: {}, ", arg.name, Type::real(arg.ty)).as_str())
                        }
                        if !s.is_empty() {
                            s.pop(); // erase ' '
                            s.pop(); // erase ','
                        }
                        s
                    },
                    if Function::get().attrs.iter().find(|x| x.as_str() == "%N").is_some() {
                        String::from(" -> !")
                    } else {
                        if ret == BuiltinType::Void.as_id() {
                            String::new()
                        } else {
                            format!(" -> {}", Type::real(ret))
                        }
                    },
                    body
                );
                Function::pop();
                x
            }
        }

        pub(in self) rule clang_case(s: &str) -> String = _ stmts:stmt()* { stmts.join(s) }

        pub rule clang() -> String = clang_case("")

        ///////////////////////////////////////////////////////////////////////////////////////////////////
        //                                     PREPROCESSOR                                              //
        ///////////////////////////////////////////////////////////////////////////////////////////////////

        rule __preprocessor_expr_one() -> String = precedence! {
            "defined" _ "(" _ name:name() _ ")" {
                Macro::is_defined(name).to_string()
            }

            "defined" __ name:name() {
                Macro::is_defined(name).to_string()
            }

            name:name() {
                Macro::find(name).unwrap_or(String::new())
            }

            any:any_except_of_newline() {
                any.to_string()
            }
        }

        rule preprocessor_expr() -> evalexpr::Value = expr:__preprocessor_expr_one()* { evalexpr::eval(expr.join("").as_str()).unwrap() }

        rule any() -> &'input str = s:$([^ '#']) { s }

        rule any_except_of_newline() -> &'input str = s:$([^ '#' | '\n']) { s }

        rule if_stmt() -> bool
            = "ifdef" ____() name:name() { Macro::is_defined(name) }
            / "ifndef" ____() name:name() { !Macro::is_defined(name) }
            / "if" ____() expr:preprocessor_expr() { boolify(expr) }

        rule else_stmt() -> String = "#" ___ "else" ___ "\n" stmts:preprocess() { stmts }

        rule elif_stmt() -> (bool, String) = "#" ___ "elif" ____() expr:preprocessor_expr() ___ "\n" stmts:preprocess() { (boolify(expr), stmts) }

        rule preprocessor_stmt() -> String = precedence! {
            "#" ___ "define" ____() name:name() ___ "\n" {
                Macro::add(name.to_string(), String::new());
                String::from("\n")
            }

            "#" ___ "define" ____() name:name() __ value:any_except_of_newline()+ "\n" {
                Macro::add(name.to_string(), value.join(""));
                String::from("\n")
            }

            "#" ___ "include" ___ path:string() ___ "\n" {
                format!("\n{}", preprocess_file(std::fs::read_to_string(path).unwrap()))
            }

            "#" ___ "undef" ____() name:name() "\n" {
                match Macro::macros().iter().enumerate().find(|(_, x)| x.name == name) {
                    Some((idx, _)) => { Macro::macros().remove(idx); },
                    None => ()
                }
                String::from("\n")
            }

            "#" ___ cond:if_stmt() ___ "\n" stmts:preprocess() elifs:elif_stmt()* other:else_stmt()? "#" ___ "endif" ___ "\n" {
                if cond {
                    format!("\n{}\n", stmts)
                } else {
                    for elif in elifs {
                        if elif.0 {
                            return format!("\n{}\n", elif.1)
                        }
                    }
                    match other {
                        Some(stmts) => format!("\n\n{}\n", stmts),
                        None => "\n".repeat(stmts.chars().fold(2, |accum, x| accum + if x == '\n' { 1 } else { 0 }))
                    }
                }
            }

            "#" ___ "\n" {
                String::from("\n")
            }

            name:name() {
                match Macro::find(name) {
                    Some(x) => x,
                    None => name.to_string()
                }
            }

            any:any() {
                any.to_string()
            }
        }

        pub rule preprocess() -> String = _ stmts:preprocessor_stmt()* { stmts.join("") }
    }
}

fn preprocess_file(mut code: String) -> String {
    Comment::uncomment(&mut code, &[
        Comment {
            begin: "//",
            end: "\n",
            save_end: true,
        },
        Comment {
            begin: "/*",
            end: "*/",
            save_end: false,
        },
    ]);

    clang::preprocess(&code).unwrap()
}

pub fn start(code: String) -> String {
    BuiltinType::add_all();
    Macro::predefine_all();

    clang::clang(&preprocess_file(code)).unwrap()
}
