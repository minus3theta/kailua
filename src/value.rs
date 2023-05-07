use std::rc::Rc;

use anyhow::bail;

use crate::vm::ExeState;

const SHORT_STR_MAX: usize = 14;
const MID_STR_MAX: usize = 48 - 1;

#[derive(Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    ShortStr(u8, [u8; SHORT_STR_MAX]),
    MidStr(Rc<(u8, [u8; MID_STR_MAX])>),
    LongStr(Rc<Vec<u8>>),
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
            (Self::ShortStr(ll, sl), Self::ShortStr(lr, sr)) => ll == lr && sl == sr,
            (Self::Function(l), Self::Function(r)) => std::ptr::eq(l, r),
            _ => false,
        }
    }
}
