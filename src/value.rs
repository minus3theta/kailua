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

impl From<String> for Value {
    fn from(s: String) -> Self {
        let len = s.len();
        if len <= SHORT_STR_MAX {
            let mut buf = [0; SHORT_STR_MAX];
            buf[..len].copy_from_slice(s.as_bytes());
            Self::ShortStr(len as u8, buf)
        } else if len <= MID_STR_MAX {
            let mut buf = [0; MID_STR_MAX];
            buf[..len].copy_from_slice(s.as_bytes());
            Self::MidStr(Rc::new((len as u8, buf)))
        } else {
            Value::LongStr(Rc::new(s.into_bytes()))
        }
    }
}

impl<'a> TryFrom<&'a Value> for &'a str {
    type Error = anyhow::Error;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        Ok(match value {
            Value::ShortStr(len, buf) => std::str::from_utf8(&buf[..*len as usize])?,
            Value::MidStr(s) => std::str::from_utf8(&s.1[..s.0 as usize])?,
            Value::LongStr(s) => std::str::from_utf8(s.as_ref())?,
            _ => bail!("not a string"),
        })
    }
}

impl TryFrom<&Value> for String {
    type Error = anyhow::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        Ok(match value {
            Value::ShortStr(len, buf) => String::from_utf8_lossy(&buf[..*len as usize]).to_string(),
            Value::MidStr(s) => String::from_utf8_lossy(&s.1[..s.0 as usize]).to_string(),
            Value::LongStr(s) => String::from_utf8_lossy(s.as_ref()).to_string(),
            _ => bail!("not a string"),
        })
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
