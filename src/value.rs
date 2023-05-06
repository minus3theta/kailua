use crate::vm::ExeState;

#[derive(Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Function(fn(&mut ExeState) -> i32),
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Integer(i) => write!(f, "{i}"),
            Self::Float(n) => write!(f, "{n:?}"),
            Self::String(s) => write!(f, "{s}"),
            Self::Function(_) => write!(f, "function"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::Boolean(l), Self::Boolean(r)) => l == r,
            (Self::Integer(l), Self::Integer(r)) => l == r,
            (Self::Float(l), Self::Float(r)) => l == r,
            (Self::String(l), Self::String(r)) => l == r,
            (Self::Function(l), Self::Function(r)) => std::ptr::eq(l, r),
            _ => false,
        }
    }
}
