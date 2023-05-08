use std::{cell::RefCell, collections::HashMap, hash::Hash, rc::Rc};

use anyhow::bail;

use crate::vm::ExeState;

const SHORT_STR_MAX: usize = 14;
const MID_STR_MAX: usize = 48 - 1;

#[derive(Clone, PartialEq)]
pub struct Table {
    pub array: Vec<Value>,
    pub map: HashMap<Value, Value>,
}

#[derive(Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    ShortStr(u8, [u8; SHORT_STR_MAX]),
    MidStr(Rc<(u8, [u8; MID_STR_MAX])>),
    LongStr(Rc<Vec<u8>>),
    Table(Rc<RefCell<Table>>),
    Function(fn(&mut ExeState) -> i32),
}

fn vec_to_short_mid_str(v: &[u8]) -> Option<Value> {
    let len = v.len();
    if len <= SHORT_STR_MAX {
        let mut buf = [0; SHORT_STR_MAX];
        buf[..len].copy_from_slice(v);
        Some(Value::ShortStr(len as u8, buf))
    } else if len <= MID_STR_MAX {
        let mut buf = [0; MID_STR_MAX];
        buf[..len].copy_from_slice(v);
        Some(Value::MidStr(Rc::new((len as u8, buf))))
    } else {
        None
    }
}

impl From<&[u8]> for Value {
    fn from(v: &[u8]) -> Self {
        vec_to_short_mid_str(v).unwrap_or_else(|| Value::LongStr(Rc::new(v.to_vec())))
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        s.as_bytes().into()
    }
}

impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Self {
        vec_to_short_mid_str(&v).unwrap_or_else(|| Value::LongStr(Rc::new(v)))
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        s.into_bytes().into()
    }
}

impl<'a> TryFrom<&'a Value> for &'a [u8] {
    type Error = anyhow::Error;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        match value {
            Value::ShortStr(len, buf) => Ok(&buf[..*len as usize]),
            Value::MidStr(s) => Ok(&s.1[..s.0 as usize]),
            Value::LongStr(s) => Ok(s),
            _ => bail!("not a string"),
        }
    }
}

impl<'a> TryFrom<&'a Value> for &'a str {
    type Error = anyhow::Error;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        Ok(std::str::from_utf8(value.try_into()?)?)
    }
}

impl TryFrom<&Value> for String {
    type Error = anyhow::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        Ok(String::from_utf8_lossy(value.try_into()?).to_string())
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Integer(i) => write!(f, "{i}"),
            Self::Float(n) => write!(f, "{n:?}"),
            Self::Table(t) => {
                let t = t.borrow();
                write!(f, "table:{}:{}", t.array.len(), t.map.len())
            }
            Self::Function(_) => write!(f, "function"),
            s => write!(f, "{}", <&str>::try_from(s).unwrap()),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Integer(i) => write!(f, "{i}"),
            Self::Float(n) => write!(f, "{n:?}"),
            Self::Table(t) => write!(f, "table: {:?}", Rc::as_ptr(t)),
            Self::Function(_) => write!(f, "function"),
            s => write!(f, "{}", <&str>::try_from(s).unwrap()),
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
            (&Self::ShortStr(ll, sl), &Self::ShortStr(lr, sr)) => {
                sl[..ll as usize] == sr[..lr as usize]
            }
            (Self::MidStr(l), Self::MidStr(r)) => l.1[..l.0 as usize] == r.1[..r.0 as usize],
            (Self::LongStr(l), Self::LongStr(r)) => *l == *r,
            (Self::Table(l), Self::Table(r)) => l == r,
            (Self::Function(l), Self::Function(r)) => std::ptr::eq(l, r),
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Value::Nil => (),
            Value::Boolean(b) => b.hash(state),
            Value::Integer(i) => i.hash(state),
            Value::Float(f) => f.to_bits().hash(state),
            Value::ShortStr(len, buf) => buf[..*len as usize].hash(state),
            Value::MidStr(s) => s.1[..s.0 as usize].hash(state),
            Value::LongStr(s) => s.hash(state),
            Value::Table(t) => Rc::as_ptr(t).hash(state),
            Value::Function(f) => (*f as *const usize).hash(state),
        }
    }
}
