//! A property value. C# stored object; the Rust 3.x line
//! collapsed that to String. The kinds are back.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
pub enum Prop {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
}

impl Prop {
    pub fn from_bool(v: bool) -> Prop {
        Prop::Bool(v)
    }

    pub fn from_int(v: i64) -> Prop {
        Prop::Int(v)
    }

    pub fn from_float(v: f64) -> Prop {
        Prop::Float(v)
    }

    pub fn from_str(v: &str) -> Prop {
        Prop::Str(v.to_string())
    }

    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            Prop::Bool(b) => Some(b),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match *self {
            Prop::Int(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match *self {
            Prop::Float(n) => Some(n),
            Prop::Int(n) => Some(n as f64),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match *self {
            Prop::Str(ref s) => Some(&s[..]),
            _ => None,
        }
    }

    /// Display form. Not an identity. Int 1 is "1";
    /// that does not make it equal to Str("1").
    pub fn as_display(&self) -> String {
        format!("{}", self)
    }

    pub fn tag(&self) -> u8 {
        match *self {
            Prop::Bool(_) => 0,
            Prop::Int(_) => 1,
            Prop::Float(_) => 2,
            Prop::Str(_) => 3,
        }
    }
}

impl PartialEq for Prop {
    fn eq(&self, other: &Prop) -> bool {
        match (self, other) {
            (&Prop::Bool(a), &Prop::Bool(b)) => a == b,
            (&Prop::Int(a), &Prop::Int(b)) => a == b,
            (&Prop::Float(a), &Prop::Float(b)) => a.to_bits() == b.to_bits(),
            (&Prop::Str(ref a), &Prop::Str(ref b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Prop {}

impl Hash for Prop {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tag().hash(state);
        match *self {
            Prop::Bool(b) => b.hash(state),
            Prop::Int(n) => n.hash(state),
            Prop::Float(n) => n.to_bits().hash(state),
            Prop::Str(ref s) => s.hash(state),
        }
    }
}

impl PartialOrd for Prop {
    fn partial_cmp(&self, other: &Prop) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Prop {
    fn cmp(&self, other: &Prop) -> Ordering {
        let t = self.tag().cmp(&other.tag());
        if t != Ordering::Equal {
            return t;
        }
        match (self, other) {
            (&Prop::Bool(a), &Prop::Bool(b)) => a.cmp(&b),
            (&Prop::Int(a), &Prop::Int(b)) => a.cmp(&b),
            (&Prop::Float(a), &Prop::Float(b)) => a.to_bits().cmp(&b.to_bits()),
            (&Prop::Str(ref a), &Prop::Str(ref b)) => a.cmp(b),
            _ => Ordering::Equal,
        }
    }
}

impl fmt::Display for Prop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Prop::Bool(true) => write!(f, "true"),
            Prop::Bool(false) => write!(f, "false"),
            Prop::Int(n) => write!(f, "{}", n),
            Prop::Float(n) => write!(f, "{}", n),
            Prop::Str(ref s) => write!(f, "{}", s),
        }
    }
}
