use crate::vm::ExeState;

#[derive(Clone)]
pub enum Value {
    Nil,
    String(String),
    Function(fn(&mut ExeState) -> i32),
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::String(s) => write!(f, "{s}"),
            Self::Function(_) => write!(f, "function"),
        }
    }
}
