use n1_tool::Config;
use serde::Deserialize;

use crate::op::Json;
use crate::{DTO, OpRequest, OpServer, Result, errs};

#[derive(Debug, Deserialize)]
pub struct MkdirDTO {
    // parent: u32,
    name: String,
}

impl DTO for MkdirDTO {
    fn check(&self) -> Result<()> {
        if self.name.is_empty() {
            errs::FIELD_EMPTY.push("field 'name'")?;
        }
        Ok(())
    }
}

pub async fn mkdir<C: Config>(_server: &OpServer<C>, _r: OpRequest<MkdirDTO>) -> Result<Json<u32>> {
    Ok(Json(42))
}
