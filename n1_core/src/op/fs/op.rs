use n1_tool::DB;
use serde::Deserialize;

use crate::{DTO, OpRequest, Result, errs};

#[derive(Debug, Deserialize)]
pub struct MkdirDTO {
    // parent: u32,
    name: String,
}

impl DTO for MkdirDTO {
    fn check(&self) -> Result<()> {
        if self.name.is_empty() {
            errs::FIELD_EMPTY.push_result("field 'name'")?;
        }
        Ok(())
    }
}

pub async fn mkdir(_: &impl DB, _r: OpRequest<MkdirDTO>) -> Result<u32> {
    Ok(42)
}
