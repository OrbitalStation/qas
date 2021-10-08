use super::{Type, TypeID, BuiltinType, Expr, Dominant};

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
#[allow(dead_code)]
pub enum Binop {
    /// a + b
    Add,

    /// a - b
    Sub,

    /// a * b
    Mul,

    /// a / b
    Div,

    /// a % b
    Rem,

    /// a && b
    And,

    /// a || b
    Or,

    /// a > b
    Greater,

    /// a < b
    Less,

    /// a >= b
    GreaterEq,

    /// a <= b
    LessEq,

    /// a == b
    Eq,

    /// a != b
    NotEq,

    /// a >> b
    RShift,

    /// a << b
    LShift,

    /// a & b
    BitAnd,

    /// a | b
    BitOr,

    /// a ^ b
    BitXor,

    Count
}

impl Binop {
    pub fn action(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Sub => "subtract",
            Self::Mul => "multiply",
            Self::Div => "divide",
            Self::Rem => "take remainder of",
            Self::And => "`and`",
            Self::Or =>  "`or`",
            Self::Greater | Self::Less | Self::GreaterEq | Self::LessEq | Self::Eq | Self::NotEq => "detect attitude",
            Self::RShift | Self::LShift => "shift",
            Self::BitAnd | Self::BitOr | Self::BitXor => "use bit operation on",
            Self::Count => unreachable!()
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::Rem => "%",
            Self::And => "&&",
            Self::Or =>  "||",
            Self::Greater => ">",
            Self::Less => "<",
            Self::GreaterEq => ">=",
            Self::LessEq => "<=",
            Self::Eq => "==",
            Self::NotEq => "!=",
            Self::RShift => ">>",
            Self::LShift => "<<",
            Self::BitAnd => "&",
            Self::BitOr => "|",
            Self::BitXor => "^",
            Self::Count => unreachable!()
        }
    }

    pub fn operate(self, mut x: Expr, mut y: Expr) -> Expr {
        if !BuiltinType::is_builtin(x.ty) || !BuiltinType::is_builtin(y.ty) || x.ty == BuiltinType::Void.as_id() || y.ty == BuiltinType::Void.as_id() {
            panic!("Cannot {} {} and {}", self.action(), Type::raw(x.ty), Type::raw(y.ty))
        }

        let mut full = |f: fn(TypeID) -> bool, default: TypeID, result_ty: fn(TypeID) -> TypeID| {
            let dominant = |x: &mut Expr, y: &mut Expr| {
                if !f(x.ty) {
                    if f(y.ty) {
                        x.convert(y.ty);
                    } else {
                        x.convert(default);
                        y.convert(default)
                    }
                } else {
                    y.convert(x.ty)
                }
            };

            match BuiltinType::dominant(x.ty, y.ty) {
                Dominant::Similar => if !f(x.ty) {
                    x.convert(default);
                    y.convert(default)
                },
                Dominant::A => dominant(&mut x, &mut y),
                Dominant::B => dominant(&mut y, &mut x)
            }

            Expr::new(format!("{} {} {}", x.name, self.as_str(), y.name), result_ty(x.ty))
        };

        const INT: TypeID = BuiltinType::SignedInt.as_id();
        const BOOL: TypeID = BuiltinType::Bool.as_id();

        match self {
            Self::Rem | Self::RShift | Self::LShift | Self::BitAnd | Self::BitOr | Self::BitXor => full(BuiltinType::is_integer, INT, |x| x),
            Self::And | Self::Or => full(|x| x == BOOL, BOOL, |x| x),
            Self::Greater | Self::Less | Self::GreaterEq | Self::LessEq | Self::Eq | Self::NotEq => full(BuiltinType::is_arithmetic, INT, |_| BOOL),
            _ => full(BuiltinType::is_arithmetic, INT, |x| x)
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
#[allow(dead_code)]
pub enum Oneop {
    /// +a
    Pos,

    /// -a
    Neg,

    /// !a
    Not,

    /// ~a
    BitNot,

    Count
}

impl Oneop {
    pub fn action(self) -> &'static str {
        match self {
            Self::Pos => "use `+` on",
            Self::Neg => "negate",
            Self::Not => "inverse",
            Self::BitNot => "inverse bits",
            Self::Count => unreachable!()
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pos => "+",
            Self::Neg => "-",
            Self::Not => "!",
            Self::BitNot => "~",
            Self::Count => unreachable!()
        }
    }

    pub fn operate(self, mut x: Expr) -> Expr {
        if !BuiltinType::is_builtin(x.ty) || x.ty == BuiltinType::Void.as_id() {
            panic!("Cannot {} {}", self.action(), Type::raw(x.ty))
        }

        let mut full = |f: fn(TypeID) -> bool, default: TypeID| {
            if !f(x.ty) {
                x.convert(default)
            }
            Expr::new(format!("{}{}", self.as_str(), x.name), x.ty)
        };

        const INT: TypeID = BuiltinType::SignedInt.as_id();
        const BOOL: TypeID = BuiltinType::Bool.as_id();

        match self {
            Self::Not => full(|x| x == BOOL, BOOL),
            Self::BitNot => full(BuiltinType::is_integer, INT),
            _ => full(BuiltinType::is_arithmetic, INT)
        }
    }
}
