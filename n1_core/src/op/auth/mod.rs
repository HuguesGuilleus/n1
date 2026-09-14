use n1_tool::Config;
use serde::Deserialize;

use crate::op::Json;
use crate::{DTO, OpRequest, OpServer, Result, errs};

pub async fn auth_test_id(_server: &OpServer<impl Config>, r: OpRequest<()>) -> Result<Json<u32>> {
    Ok(Json(r.token.uid))
}

pub async fn auth_test_isadmin(
    _server: &OpServer<impl Config>,
    r: OpRequest<()>,
) -> Result<Json<bool>> {
    Ok(Json(r.token.is_admin))
}

#[derive(Debug, Deserialize)]
pub struct AccessDTO {
    pub id: u32,
    pub app: u16,
    pub can_write: bool,
}

impl DTO for AccessDTO {
    fn check(&self) -> Result<()> {
        if self.id == 0 {
            errs::FIELD_EMPTY.push(format!("integer field id is zero"))?;
        }
        if self.app == 0 {
            errs::FIELD_EMPTY.push(format!("integer field app is zero"))?;
        }
        Ok(())
    }
}

pub async fn auth_test_access(
    _server: &OpServer<impl Config>,
    r: OpRequest<AccessDTO>,
) -> Result<Json<bool>> {
    Ok(Json(if r.dto.can_write {
        r.token.check_access_write(r.dto.id, r.dto.app).is_ok()
    } else {
        r.token.check_access_read(r.dto.id, r.dto.app).is_ok()
    }))
}
