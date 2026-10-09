use n1_tool::DB;
use serde::Deserialize;

use crate::{DTO, OpRequest, OpServer, Result, errs};

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

pub async fn mkdir<C: DB>(_server: &OpServer<C>, _r: OpRequest<MkdirDTO>) -> Result<u32> {
    Ok(42)
}
