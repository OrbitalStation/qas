use super::{Expr, BuiltinType, Function, TypeID, Dominant, FnFlags, BitPtrIter};

pub struct Op {
    pub name: String,
    pub matching: fn(&TypeID) -> bool,
    pub default: TypeID,
    pub result: fn(&TypeID) -> TypeID,
    pub operands: u8
}

impl Op {
    pub fn add_all() {
        let int = BuiltinType::SignedInt.as_id();
        let uint = BuiltinType::UnsignedInt.as_id();
        let r#bool = BuiltinType::Bool.as_id();
        let same = BuiltinType::Count.as_id();

        fn always_bool(_: &TypeID) -> TypeID { BuiltinType::Bool.as_id() }

        fn is_bool(ty: &TypeID) -> bool { *ty == BuiltinType::Bool.as_id() }
        fn is_any(_: &TypeID) -> bool { true }
        fn is_mutable_integer(x: &TypeID) -> bool {
            if !x.mutable {
                panic!("cannot apply operator to const value")
            }
            BuiltinType::is_integer(x)
        }

        /* binary */

        Self::add("*", 2, BuiltinType::is_arithmetic, int.clone(), Clone::clone);
        Self::add("/", 2, BuiltinType::is_arithmetic, int.clone(), Clone::clone);
        Self::add("%", 2, BuiltinType::is_integer, int.clone(), Clone::clone);

        Self::add("+", 2, BuiltinType::is_arithmetic, int.clone(), Clone::clone);
        Self::add("-", 2, BuiltinType::is_arithmetic, int.clone(), Clone::clone);

        Self::add("<<", 2, BuiltinType::is_unsigned, uint.clone(), Clone::clone);
        Self::add(">>", 2, BuiltinType::is_unsigned, uint.clone(), Clone::clone);

        Self::add(">", 2, BuiltinType::is_builtin, same.clone(), always_bool);
        Self::add("<", 2, BuiltinType::is_builtin, same.clone(), always_bool);
        Self::add(">=", 2, BuiltinType::is_builtin, same.clone(), always_bool);
        Self::add("<=", 2, BuiltinType::is_builtin, same.clone(), always_bool);

        Self::add("==", 2, BuiltinType::is_builtin, same.clone(), always_bool);
        Self::add("!=", 2, BuiltinType::is_builtin, same.clone(), always_bool);

        Self::add("&", 2, BuiltinType::is_unsigned, uint.clone(), Clone::clone);

        Self::add("^", 2, BuiltinType::is_unsigned, uint.clone(), Clone::clone);

        Self::add("|", 2, BuiltinType::is_unsigned, uint.clone(), Clone::clone);

        Self::add("&&", 2, is_bool, r#bool.clone(), Clone::clone);

        Self::add("||", 2, is_bool, r#bool.clone(), Clone::clone);

        /* unary */

        // ++ postfix(a = after)
        Self::add("++a", 1, is_mutable_integer, same.clone(), Clone::clone);
        // ++ prefix(b = before)
        Self::add("++b", 1, is_mutable_integer, same.clone(), Clone::clone);

        // -- postfix(a = after)
        Self::add("--a", 1, is_mutable_integer, same.clone(), Clone::clone);
        // -- prefix(b = before)
        Self::add("--b", 1, is_mutable_integer, same.clone(), Clone::clone);
        Self::add("+", 1, BuiltinType::is_arithmetic, int.clone(), Clone::clone);
        Self::add("-", 1, BuiltinType::is_arithmetic, int.clone(), Clone::clone);
        Self::add("&", 1, is_any, same.clone(), |x| TypeID {
            idx: x.idx,
            ptr: BitPtrIter::append(x.ptr.clone(), x.mutable),
            mutable: true
        });
        Self::add("*", 1, |x| {
            if x.idx == 0 {
                panic!("cannot deref non-ptr")
            } else if x.ptr.len() == 1 && x.idx == BuiltinType::Void as usize {
                panic!("cannot deref `void *`")
            }
            true
        }, same.clone(), |x| {
            let mut x = x.clone();
            let mutable = x.ptr.pop().unwrap();
            TypeID {
                idx: x.idx,
                ptr: x.ptr,
                mutable
            }
        });
        Self::add("!", 1, is_bool, r#bool.clone(), Clone::clone);
        Self::add("~", 1, BuiltinType::is_unsigned, uint.clone(), Clone::clone);
    }

    pub fn add(name: &str, operands: u8, matching: fn(&TypeID) -> bool, default: TypeID, result: fn(&TypeID) -> TypeID) {
        Self::ops().push(Self {
            name: name.to_string(),
            matching,
            default,
            result,
            operands
        })
    }

    #[inline(always)]
    pub fn ops() -> &'static mut Vec <Op> {
        static mut OPS: Vec <Op> = Vec::new();
        unsafe { &mut OPS }
    }

    pub fn find(name: &str, ops: u8) -> usize {
        let mut i = 0;
        while i < Self::ops().len() {
            if Self::ops()[i].name == name && Self::ops()[i].operands == ops { return i }
            i += 1
        }
        unreachable!()
    }
}

pub struct Operand {
    pub op: usize,
    pub expr: Expr
}

impl Operand {
    pub fn new(name: &str, expr: Expr, ops: u8) -> Self {
        Self {
            op: Op::find(name, ops),
            expr
        }
    }

    pub fn op(&self) -> &'static Op {
        &Op::ops()[self.op]
    }

    pub fn parse_to_num(i: String) -> Expr {
        let has_dot = i.chars().find(|x| *x == '.').is_some();

        Expr::new(i, &if has_dot {
            BuiltinType::Float.as_id()
        } else {
            BuiltinType::UnsignedChar.as_id()
        })
    }

    pub fn parse_to_var(i: &str) -> Expr {
        Expr::new(i.to_string(), &Function::get().type_of_let(&i).expect("unknown variable"))
    }
}

pub struct Binop;

impl Binop {
    fn dominant(x: &mut Expr, y: &mut Expr, matching: fn(&TypeID) -> bool, default: &TypeID) {
        if !matching(&x.ty) {
            if matching(&y.ty) {
                x.convert(&y.ty);
            } else {
                x.convert(&default);
                y.convert(&default)
            }
        } else {
            y.convert(&x.ty)
        }
    }

    pub fn union(mut x: Expr, ops: Vec <Operand>) -> Expr {
        if ops.is_empty() { return x }

        for mut op in ops {
            let matching = op.op().matching;
            let default = &op.op().default;

            match BuiltinType::dominant(&x.ty, &op.expr.ty) {
                Dominant::Similar => if !matching(&x.ty) {
                    x.convert(default);
                    op.expr.convert(default)
                },
                Dominant::A => Self::dominant(&mut x, &mut op.expr, matching, default),
                Dominant::B => Self::dominant(&mut op.expr, &mut x, matching, default)
            }

            x = Expr::new(format!("{} {} {}", x.name, op.op().name, op.expr.name), &(op.op().result)(&x.ty))
        }

        x
    }
}

pub struct Unop;

impl Unop {
    pub fn union(mut x: Expr, operator: &str) -> Expr {
        let op = &Op::ops()[Op::find(operator, 1)];

        if !(op.matching)(&x.ty) {
            x.convert(&op.default)
        }

        let mut content = format!("{}{}", if operator == "~" {
            "!"
        } else {
            operator
        }, x.name);

        if op.name == "*" {
            if Function::get().should_be_safe() {
                content = format!("unsafe {{ {} }}", content)
            } else {
                Function::get().flags.remove(FnFlags::SAFE)
            }
        } else if op.name == "++a" {
            Function::check_and_make_mutable_on_require(&x.name);
            content = format!("::qas::builtin::inca(&mut {})", x.name)
        } else if op.name == "++b" {
            Function::check_and_make_mutable_on_require(&x.name);
            content = format!("::qas::builtin::incb(&mut {})", x.name)
        } else if op.name == "--a" {
            Function::check_and_make_mutable_on_require(&x.name);
            content = format!("::qas::builtin::deca(&mut {})", x.name)
        } else if op.name == "--b" {
            Function::check_and_make_mutable_on_require(&x.name);
            content = format!("::qas::builtin::decb(&mut {})", x.name)
        }

        Expr::new(content, &(op.result)(&x.ty))
    }
}

//         match self {
//             Self::Deref => if x.ty.ptr != 0 && x.ty.idx != BuiltinType::Void as usize {
//                 Expr::new(format!("{}", if Function::get().should_be_safe() {
//                     format!("unsafe {{ *{} }}", x.name)
//                 } else {
//                     Function::get().flags.remove(FnFlags::SAFE);
//                     format!("*{}", x.name)
//                 }), &TypeID {
//                     idx: x.ty.idx,
//                     ptr: x.ty.ptr - 1,
//                     mutable: x.ty.mutable
//                 })
//             } else {
//                 self.panic(&x.ty)
//             },
//             Self::Ref => Expr::new(format!("&{}", x.name), &TypeID {
//                 idx: x.ty.idx,
//                 ptr: x.ty.ptr + 1,
//                 mutable: x.ty.mutable
//             }),
//             Self::Not => full(|x| *x == BOOL, BOOL),
//             Self::BitNot => full(BuiltinType::is_integer, INT),
//             Self::Pos => {
//                 if !BuiltinType::is_arithmetic(&x.ty) {
//                     x.convert(&INT);
//                 }
//                 Expr::new(format!("{}", x.name), &x.ty)
//             },
//             _ => full(BuiltinType::is_arithmetic, INT)
//         }
//     }
// }
