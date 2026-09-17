use std::fmt::Debug;

use serde::{Deserialize, de::DeserializeOwned};

use crate::{Result, errs};

pub trait DTO: DeserializeOwned + Debug {
    fn check(&self) -> Result<()>;
}

pub trait URLDTO: DTO {
    /// Decode information from path.
    /// Example: in `/_wiki_page/1/2` decode `1/2`
    fn url_decode(url: &str) -> Result<Self>;
}

impl DTO for () {
    fn check(&self) -> Result<()> {
        Ok(())
    }
}
impl URLDTO for () {
    fn url_decode(url: &str) -> Result<Self> {
        if url != "" {
            return Err(errs::FIELD_NOT_EMPTY.into());
        }
        Ok(())
    }
}

impl DTO for u32 {
    fn check(&self) -> Result<()> {
        if *self == 0 {
            return Err(errs::FIELD_ID_ZERO.into());
        }
        Ok(())
    }
}
impl URLDTO for u32 {
    fn url_decode(url: &str) -> Result<Self> {
        url.parse::<u32>().map_err(|_| errs::DECODE_REQUEST.into())
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct EntityAndObjectDTO {
    pub eid: u32,
    pub oid: u32,
}
impl DTO for EntityAndObjectDTO {
    fn check(&self) -> Result<()> {
        if self.eid == 0 {
            return Err(errs::FIELD_ENTITY.into());
        }
        if self.oid == 0 {
            return Err(errs::FIELD_OID.into());
        }
        Ok(())
    }
}
impl URLDTO for EntityAndObjectDTO {
    fn url_decode(url: &str) -> Result<Self> {
        let (a, b) = url.split_once(',').ok_or(errs::DECODE_REQUEST)?;
        Ok(Self {
            eid: a.parse::<u32>().map_err(|_| errs::DECODE_REQUEST)?,
            oid: b.parse::<u32>().map_err(|_| errs::DECODE_REQUEST)?,
        })
    }
}
impl n1_html::Html for EntityAndObjectDTO {
    fn render(&self, buf: &mut String) {
        use std::fmt::Write;
        write!(buf, "{},{}", self.eid, self.oid).unwrap()
    }
}
