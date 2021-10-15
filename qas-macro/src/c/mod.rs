mod builtin;
mod fns;
mod op;
mod preprocessor;
mod ty;
mod comment;

use crate::StringExt;
use check_keyword::CheckKeyword;

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
    pub fn new(name: String, ty: &TypeID) -> Self {
        Self { name, ty: ty.clone() }
    }

    pub fn convert(&mut self, to: &TypeID) {
        self.name = BuiltinType::convert(&self.ty, &to, &self.name);
        self.ty = to.clone()
    }
}

pub struct BitPtrIter;

impl BitPtrIter {
    pub fn append(mut vec: bit_vec::BitVec <u8>, value: bool) -> bit_vec::BitVec <u8> {
        vec.push(value);
        vec
    }

    pub fn append_many(mut vec: bit_vec::BitVec <u8>, value: Vec <bool>) -> bit_vec::BitVec <u8> {
        vec.extend(value.into_iter());
        vec
    }
}

peg::parser! { pub grammar clang() for str {
    rule digit2() -> &'input str
        = x:$(['0' | '1'])

    rule digit8() -> &'input str
        = x:$(['0'..='7'])

    rule digit10() -> &'input str
        = x:$(['0'..='9'])

    rule digit16() -> &'input str
        = x:$(['0'..='9' | 'a'..='f' | 'A'..='F'])

    rule letter() -> &'input str
        = x:$(['a'..='z' | 'A'..='Z' | '_' | '$'])

    rule name() -> &'input str
        = x:$(letter() (letter() / digit10())*) { x }

    rule __num_suffix() -> &'static str
        = $("l" / "L" / "ll" / "LL") { "i64" }
        / $("u" / "U") { "u64" }

    rule num() -> String
        = "0b" num:$(digit2()+) suffix:__num_suffix()? { String::from("0b") + num + suffix.unwrap_or_default() }
        / "0o" num:$(digit8()+) suffix:__num_suffix()? { String::from("0o") + num + suffix.unwrap_or_default() }
        / "0x" num:$(digit16()+) suffix:__num_suffix()? { String::from("0x") + num + suffix.unwrap_or_default() }
        / num:$(digit10()+ "."? digit10()*) suffix:__num_suffix()? { num.to_string() + suffix.unwrap_or_default() }

    rule punct() -> &'input str
        = x:$(['+' | '-' | '*' | '/' | '%' | '&' | '|' | '=' | '<' | '>' | '!' | '^' | '~']*)

    rule __string_suffix() -> bool
        = x:"L"? { x.is_none() }

    rule __string_one(ext: bool) -> String = precedence! {
        "\\\"" { String::from("\"") }
        x:$([^ '\"']) {
            if !x.is_ascii() && ext {
                panic!("non-ascii character are not allowed in strings; try using `L` suffix to support it")
            }
            x.to_string()
        }
    }

    rule string() -> String
        = suffix:__string_suffix() "\"" x:__string_one(suffix)* "\"" { x.join("") + "\0" }

    rule newline() = "\n" {
        unsafe { LINE += 1 }
    }

    rule ___unit()
        = [' ' | '\t']
        / newline()

    rule _ = ___unit()*

    rule __ = ___unit()+

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

    pub(in self) rule try_ty() -> Result <usize, &'input str> = precedence! {
        x:__builtin_try_ty() { Ok(x as usize) }

        x:name() {
            let mut i = BuiltinType::Count as usize;
            while i < FullType::types().len() {
                if FullType::raw(&TypeID::from(i, true)) == x { return Ok(i) }
                i += 1
            }

            Err(x)
        }
    }

    rule __ty_ptr() -> bool = _ "*" c:$(_ "const")? { c.is_none() }

    rule ty() -> TypeID = x:$("const" __)? ty:try_ty() ptr:__ty_ptr()* _ {?
        match ty {
            Ok(idx) => Ok(TypeID {
                idx,
                ptr: BitPtrIter::append_many(Default::default(), ptr),
                mutable: x.is_none()
            }),
            Err(name) => {
                match AliasType::find(name) {
                    Some(id) => Ok(TypeID {
                        idx: id.idx,
                        ptr: BitPtrIter::append_many(id.ptr.clone(), ptr),
                        mutable: id.mutable && x.is_none()
                    }),
                    None => Err("expected type")
                }
            }
        }
    }

    rule var() -> &'input str = x:name() {?
        match try_ty(&x) {
            Ok(id) => match id {
                Ok(_) => Err(""),
                Err(_) => Ok(x)
            },
            Err(_) => Err("")
        }
    }

    rule __expr_arg() -> Expr = "," _ expr:expr() _ {
        expr
    }

    rule __sizeof() -> TypeID
        = __ ty:ty() { ty }
        / _ "(" _ ty:ty() _ ")" { ty }

    rule _e_num() -> Expr = i:num() {
        Operand::parse_to_num(i)
    }

    rule _e_var() -> Expr = i:var() {
        Operand::parse_to_var(i)
    }

    rule _e_unop() -> Expr = precedence! {
        "sizeof" ty:__sizeof() {
            Expr::new(format!("::qas::builtin::sizeof::<{}>()", FullType::real(&ty)), &BuiltinType::usized().as_id())
        }

        i:_e_e0() _ "++" {
            Unop::union(i, "++a")
        }

        i:_e_e0() _ "--" {
            Unop::union(i, "--a")
        }

        "++" _ i:_e_e0() {
            Unop::union(i, "++b")
        }

        "--" _ i:_e_e0() {
            Unop::union(i, "--b")
        }

        "+" _ i:_e_e1() {
            Unop::union(i, "")
        }

        "-" _ i:_e_e1() {
            Unop::union(i, "-")
        }

        "&" _ i:_e_e0() {
            Unop::union(i, "&")
        }

        "*" _ i:_e_e1() {
            Unop::union(i, "*")
        }

        "!" _ i:_e_e1() {
            Unop::union(i, "!")
        }

        "~" _ i:_e_e1() {
            Unop::union(i, "~")
        }
    }

    rule _e_parens() -> Expr = "(" _ i:expr() _ ")" {
        Expr::new(i.name.parentify(), &i.ty)
    }

    rule _e_e0() -> Expr
        = name:var() _ "(" _ arg0:expr()? _ args:__expr_arg()* ("," _)? ")" {
            let f = &Function::fns().iter().find(|x| x.name == name).map(|x| x.as_builtin()).or_else(|| BuiltinFunction::fns().iter().find(|x| x.name == name).map(Clone::clone)).expect("unknown function");
            let mut args = args;

            match arg0 {
                Some(x) => args.insert(0, x),
                None => if args.len() != 0 {
                    panic!("unexpected comma")
                }
            }

            if args.len() > f.args.len() {
                panic!("too many args")
            } else if args.len() < f.args.len() {
                panic!("lack of args")
            }

            Expr::new(format!("{}({})", f.real, {
                let mut s = String::new();
                let mut i = 0;
                while i < args.len() {
                    s.push_str(format!("{}, ", BuiltinType::convert(&args[i].ty, f.args[i], &args[i].name)).as_str());
                    i += 1
                }
                if !s.is_empty() {
                    s.pop(); // erase ' '
                    s.pop(); // erase ','
                }
                s
            }), &f.ret)
        }
        / i:_e_num() { i }
        / i:string() {
            Expr::new(format!("\"{}\"", i), &TypeID {
                idx: BuiltinType::UnsignedChar as usize,
                ptr: BitPtrIter::append(Default::default(), false),
                mutable: false
            })
        }
        / "__func__" {
            Expr::new(format!("\"{}\0\"", Function::get().name), &TypeID {
                idx: BuiltinType::UnsignedChar as usize,
                ptr: BitPtrIter::append(Default::default(), false),
                mutable: false
            })
        }
        / i:_e_var() { i }

    rule _e_e1() -> Expr
        = i:_e_parens() { i }
        / i:_e_unop() { i }
        / i:_e_e0()  { i }

    rule _e_o1() -> Operand = _ op:$("/" / "*" / "%") _ i:_e_e1() {
        Operand::new(op, i, 2)
    }

    rule _e_e2() -> Expr = x:_e_e1() ops:_e_o1()* {
        Binop::union(x, ops)
    }

    rule _e_o2() -> Operand = _ op:$("-" / "+") _ i:_e_e2() {
        Operand::new(op, i, 2)
    }

    rule _e_e3() -> Expr = x:_e_e2() ops:_e_o2()* {
        Binop::union(x, ops)
    }

    rule _e_o3() -> Operand = _ op:$("<<" / ">>") _ i:_e_e3() {
        Operand::new(op, i, 2)
    }

    rule _e_e4() -> Expr = x:_e_e3() ops:_e_o3()* {
        Binop::union(x, ops)
    }

    rule _e_o4() -> Operand = _ op:$("<" / ">" / "<=" / ">=") _ i:_e_e4() {
        Operand::new(op, i, 2)
    }

    rule _e_e5() -> Expr = x:_e_e4() ops:_e_o4()* {
        Binop::union(x, ops)
    }

    rule _e_o5() -> Operand = _ op:$("==" / "!=") _ i:_e_e5() {
        Operand::new(op, i, 2)
    }

    rule _e_e6() -> Expr = x:_e_e5() ops:_e_o5()* {
        Binop::union(x, ops)
    }

    rule _e_o6() -> Operand = _ op:$("&") _ i:_e_e6() {
        Operand::new(op, i, 2)
    }

    rule _e_e7() -> Expr = x:_e_e6() ops:_e_o6()* {
        Binop::union(x, ops)
    }

    rule _e_o7() -> Operand = _ op:$("^") _ i:_e_e7() {
        Operand::new(op, i, 2)
    }

    rule _e_e8() -> Expr = x:_e_e7() ops:_e_o7()* {
        Binop::union(x, ops)
    }

    rule _e_o8() -> Operand = _ op:$("|") _ i:_e_e8() {
        Operand::new(op, i, 2)
    }

    rule _e_e9() -> Expr = x:_e_e8() ops:_e_o8()* {
        Binop::union(x, ops)
    }

    rule _e_o9() -> Operand = _ op:$("&&") _ i:_e_e9() {
        Operand::new(op, i, 2)
    }

    rule _e_eA() -> Expr = x:_e_e9() ops:_e_o9()* {
        Binop::union(x, ops)
    }

    rule _e_oA() -> Operand = _ op:$("||") _ i:_e_eA() {
        Operand::new(op, i, 2)
    }

    rule _e_eB() -> Expr = x:_e_eA() ops:_e_oA()* {
        Binop::union(x, ops)
    }

    rule expr() -> Expr = precedence! {
        cond:(@) _ "?" _ s1:expr() _ ":" _ s2:@ {
            let mut s1 = s1;
            let mut s2 = s2;

            match BuiltinType::dominant(&s1.ty, &s2.ty) {
                Dominant::Similar => (),
                Dominant::A => s2.convert(&s1.ty),
                Dominant::B => s1.convert(&s2.ty)
            }
            let cond = BuiltinType::convert(&cond.ty, &BuiltinType::Bool.as_id(), &cond.name).deparentify();
            Expr::new(format!("if {} {{\n{tabs}\t{}\n{tabs}}} else {{\n\t{tabs}{}\n\t}}", cond, s1.name, s2.name, tabs = Tab::tabs()), &s1.ty)
        }

        x:_e_eB() {
            Expr::new(x.name.deparentify(), &x.ty)
        }
    }

    rule __stmt_add(attrs: Vec <String>, name: &str, ret: TypeID, arg0: Option <(TypeID, &str)>, args: Vec <Let>) = _ {
        let mut args = args;
        let mut attrs = attrs;

        match arg0 {
            Some(x) => args.insert(0, Let {
                name: x.1.to_string(),
                mutable: false,
                ty: x.0
            }),
            None => if args.len() != 0 {
                panic!("unexpected comma")
            }
        }

        if (1..attrs.len()).any(|i| attrs[i..].contains(&attrs[i - 1])) {
            panic!("cannot have duplicate attributes")
        }

        let mut flags = FnFlags::empty();
        flags.insert(FnFlags::SAFE);
        flags.insert(FnFlags::PUBLIC);

        let mut i = 0;
        let mut remove = true;
        while i < attrs.len() {
            match attrs[i].as_str() {
                "%U" => flags.remove(FnFlags::SAFE),
                "%P" => flags.remove(FnFlags::PUBLIC),
                _ => remove = false
            }

            if remove {
                attrs.remove(i);
                remove = true
            }
            i += 1
        }

        Function::add(Function {
            name: name.to_string(),
            ret,
            args: args.len(),
            lets: args,
            attrs,
            flags
        })
    }

    rule __stmt_arg() -> Let = "," _ ty:ty() _ name:var() _ {
        Let {
            name: name.to_string(),
            ty: ty.mutable(),
            mutable: false
        }
    }

    rule __stmt_fn_attr_inside() -> String
        = "\"noreturn\""                     { String::from("%N") }
        / "\"safe\""                         { String::from("%S") }
        / "\"unsafe\""                       { String::from("%U") }
        / "rust" _ "(" _ attr:string() _ ")" { attr }

    rule __stmt_fn_attr() -> String
        = "inline" _ { String::from("inline") }
        / "static" _ { String::from("%P") }
        / "__ATTR__" _ "((" _ attr:__stmt_fn_attr_inside() _ "))" _ { attr }

    rule __stmt_fn_attrs() -> Vec <String> = attrs:__stmt_fn_attr()*

    rule __stmt_fn_first_arg() -> (TypeID, &'input str) = ty:ty() _ arg:var() _ {
        (ty, arg)
    }

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
            let result = BuiltinType::convert(&e.ty, &Function::get().ret, &e.name).deparentify();
            if is_last.is_some() {
                format!("{}", result)
            } else {
                format!("return {};", result)
            }
        }

        "typedef" __ ty:ty() _ new:var() _ ";" _ {
            let real = FullType::real(&ty);
            let x = if new != real && !new.is_keyword() {
                format!("type {} = {};", new, real)
            } else {
                String::new()
            };
            AliasType::add(new.to_string(), ty);
            x
        }

        attrs:__stmt_fn_attrs() ret:ty() _ name:var() _ "(" _ arg0:__stmt_fn_first_arg()? args:__stmt_arg()* ("," _)? ")" __stmt_add(attrs, name, ret, arg0, args) "{" _ body:clang()? _ "}" _ {
            let x = format!("\n{}{}{}fn {}({}){} {{\n\t{}\n}}\n\n",
                {
                    let mut s = String::new();
                    for attr in &Function::get().attrs {
                        if attr.chars().next().unwrap() != '%' {
                            s.push_str(format!("#[{}]\n", attr).as_str())
                        }
                    }
                    s
                },
                if Function::get().flags.contains(FnFlags::PUBLIC) { "pub " } else { "" },
                if Function::get().flags.contains(FnFlags::SAFE) { "" } else { "unsafe " },
                Function::get().name,
                {
                    let mut s = String::new();
                    for arg in Function::get().lets[..Function::get().args].iter() {
                        s.push_str(format!("{}{}: {}, ", if arg.mutable {
                            "mut "
                        } else {
                            ""
                        }, arg.name, FullType::real(&arg.ty)).as_str())
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
                    if Function::get().ret == BuiltinType::Void.as_id() {
                        String::new()
                    } else {
                        format!(" -> {}", FullType::real(&Function::get().ret))
                    }
                },
                body.unwrap_or_else(|| if Function::get().ret == BuiltinType::Void.as_id() {
                    String::new()
                } else {
                    panic!("it's not allowed to have empty body for functions, which return type is non-void")
                })
            );

            // Clear everything that is connected to current function

            AliasType::clear();

            x
        }
    }

    pub rule clang() -> String = _ stmts:stmt()* { stmts.join(&Tab::tabs_nl()) }

    ///////////////////////////////////////////////////////////////////////////////////////////////////
    //                                     PREPROCESSOR                                              //
    ///////////////////////////////////////////////////////////////////////////////////////////////////

    // rule __preprocessor_expr_one() -> String = precedence! {
    //     "defined" _ "(" _ name:name() _ ")" {
    //         Macro::is_defined(name).to_string()
    //     }
    //
    //     "defined" __ name:name() {
    //         Macro::is_defined(name).to_string()
    //     }
    //
    //     name:name() {
    //         Macro::find(name).unwrap_or(String::new())
    //     }
    //
    //     any:any_except_of_newline() {
    //         any.to_string()
    //     }
    // }

    rule any_except_of_newline() -> &'input str = s:$([^ '#' | '\n']) { s }

    rule if_stmt() -> bool
        = "ifdef" ____() name:name() { Macro::is_defined(name) }
        / "ifndef" ____() name:name() { !Macro::is_defined(name) }

    rule else_stmt() -> String = "#" ___ "else" ___ newline() stmts:preprocess() { stmts }

    rule line_possible_file() -> String = ____() file:string() { file }

    rule preprocessor_stmt() -> String = precedence! {
        "#" ___ "define" ____() name:name() ___ newline() {
            Macro::add(name.to_string(), String::new());
            String::from("\n")
        }

        "#" ___ "define" ____() name:name() __ value:any_except_of_newline()+ newline() {
            Macro::add(name.to_string(), value.join(""));
            String::from("\n")
        }

        "#" ___ "include" ___ path:string() ___ newline() {
            format!("\n{}", preprocess_file(crate::read_file(&path)))
        }

        "#" ___ "undef" ____() name:name() newline() {
            match Macro::macros().iter().enumerate().find(|(_, x)| x.name == name) {
                Some((idx, _)) => { Macro::macros().remove(idx); },
                None => ()
            }
            String::from("\n")
        }

        "#" ___ cond:if_stmt() ___ newline() stmts:preprocess() other:else_stmt()? "#" ___ "endif" ___ newline() {
            if cond {
                format!("\n{}\n", stmts)
            } else {
                // for elif in elifs {
                //     if elif.0 {
                //         return format!("\n{}\n", elif.1)
                //     }
                // }
                match other {
                    Some(stmts) => format!("\n\n{}\n", stmts),
                    None => "\n".repeat(stmts.chars().fold(2, |accum, x| accum + if x == '\n' { 1 } else { 0 }))
                }
            }
        }

        "#" ___ "line" ____() line:num() file:line_possible_file()? ___ newline() {
            assert!(line.find('.').is_none(), "line cannot be float");
            unsafe {
                LINE = line.parse().expect("wrong format");
                match file {
                    Some(file) => FILE = format!("\"{}\"", file),
                    None => ()
                }
            }
            String::from("\n")
        }

        "#" ___ newline() {
            String::from("\n")
        }

        "__LINE__" {
            unsafe { LINE.to_string() }
        }

        "__FILE__" {
            unsafe { FILE.clone() }
        }

        name:name() {
            match Macro::find(name) {
                Some(x) => x,
                None => name.to_string()
            }
        }

        newline() {
            String::from("\n")
        }

        any:any_except_of_newline() {
            any.to_string()
        }
    }

    pub rule preprocess() -> String = _ stmts:preprocessor_stmt()* { stmts.join("") }
} }

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

pub fn start(file: &str, code: String) -> String {
    BuiltinType::add_all();
    BuiltinFunction::add_all();
    Op::add_all();

    Macro::predefine_all(file);
    let code = preprocess_file(code);

    match clang::clang(&code) {
        Ok(s) => s,
        Err(e) => panic!("{}", e)
    }
}
