use std::{
    fmt::{Debug, Display, Write},
    ops::Deref,
    str::FromStr,
};

use serde::{Deserialize, Serialize, de::Visitor};

use crate::{ErrorStep, errs};

/// A string stored as a fixed-size array of bytes with a length indicator.
/// Do not contain `_` or space characters.
/// It contain at min one character.
#[derive(Clone, PartialEq, Eq, PartialOrd)]
pub struct Name {
    len: usize,
    data: [u8; Self::MAX_SIZE],
}

impl Name {
    pub const MAX_SIZE: usize = 64;

    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.data[..self.len]) }
    }
}

impl Deref for Name {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl FromStr for Name {
    type Err = errs::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('_') || s.split_whitespace().count() > 1 {
            return Err(ErrorStep::NameInvalid.into());
        } else if s.len() > Self::MAX_SIZE {
            return Err(ErrorStep::NameTooLong { len: s.len() }.into());
        } else if s.is_empty() {
            return Err(ErrorStep::NameEmpty.into());
        }
        let mut data = [0u8; Self::MAX_SIZE];
        data[..s.len()].copy_from_slice(s.as_bytes());
        Ok(Self { len: s.len(), data })
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Debug for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('"')?;
        str::escape_debug(self.as_str())
            .map(|c| f.write_char(c))
            .collect::<Result<(), std::fmt::Error>>()?;
        f.write_char('"')?;
        Ok(())
    }
}

impl Serialize for Name {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(NameVisitor)
    }
}

struct NameVisitor;

impl<'de> Visitor<'de> for NameVisitor {
    type Value = Name;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a valid Name")
    }

    fn visit_str<E>(self, value: &str) -> Result<Name, E>
    where
        E: serde::de::Error,
    {
        Name::from_str(value).map_err(E::custom)
    }
}

#[test]
fn test_name() {
    let n = Name::from_str("aBc").unwrap();
    assert_eq!(n.as_str(), "aBc");
    assert_eq!(format!("{}", n), "aBc");
    assert_eq!(format!("{:?}", n), "\"aBc\"");

    assert_eq!(serde_json::to_string(&n).unwrap(), "\"aBc\"");
    assert_eq!(serde_json::from_slice::<Name>(b"\"aBc\"").unwrap(), n);
}
