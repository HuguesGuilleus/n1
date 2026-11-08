use std::fmt::Debug;

use serde::{Deserialize, de::DeserializeOwned};

use crate::{ErrorStep, Result};

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
            return Err(ErrorStep::NeedNoData.into());
        }
        Ok(())
    }
}

impl DTO for u32 {
    fn check(&self) -> Result<()> {
        if *self == 0 {
            return Err(ErrorStep::FieldZero { name: "" }.into());
        }
        Ok(())
    }
}
impl URLDTO for u32 {
    fn url_decode(url: &str) -> Result<Self> {
        url.parse::<u32>()
            .map_err(|err| ErrorStep::ParsingInteger { err }.into())
    }
}

impl DTO for String {
    fn check(&self) -> Result<()> {
        if self.is_empty() {
            return Err(ErrorStep::FieldEmpty { name: "" }.into());
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct EntityAndObjectDTO {
    pub owner: u32,
    pub object: u32,
}
impl DTO for EntityAndObjectDTO {
    fn check(&self) -> Result<()> {
        if self.owner == 0 {
            return Err(ErrorStep::FieldZero { name: "owner" }.into());
        }
        if self.object == 0 {
            return Err(ErrorStep::FieldZero { name: "object" }.into());
        }
        Ok(())
    }
}
impl URLDTO for EntityAndObjectDTO {
    fn url_decode(url: &str) -> Result<Self> {
        let (a, b) = url.split_once(',').ok_or(ErrorStep::Parsing2Integer)?;
        Ok(Self {
            owner: a
                .parse::<u32>()
                .map_err(|err| ErrorStep::ParsingInteger { err })?,
            object: b
                .parse::<u32>()
                .map_err(|err| ErrorStep::ParsingInteger { err })?,
        })
    }
}
impl n1_html::Html for EntityAndObjectDTO {
    fn render(&self, buf: &mut String) {
        use std::fmt::Write;
        write!(buf, "{},{}", self.owner, self.object).unwrap()
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct EntityAndObjectNullDTO {
    pub owner: u32,
    pub object: u32,
}
impl DTO for EntityAndObjectNullDTO {
    fn check(&self) -> Result<()> {
        if self.owner == 0 {
            return Err(ErrorStep::FieldZero { name: "owner" }.into());
        }
        Ok(())
    }
}
impl URLDTO for EntityAndObjectNullDTO {
    fn url_decode(url: &str) -> Result<Self> {
        let (a, b) = url.split_once(',').ok_or(ErrorStep::Parsing2Integer)?;
        Ok(Self {
            owner: a
                .parse::<u32>()
                .map_err(|err| ErrorStep::ParsingInteger { err })?,
            object: b
                .parse::<u32>()
                .map_err(|err| ErrorStep::ParsingInteger { err })?,
        })
    }
}
impl n1_html::Html for EntityAndObjectNullDTO {
    fn render(&self, buf: &mut String) {
        use std::fmt::Write;
        write!(buf, "{},{}", self.owner, self.object).unwrap()
    }
}
