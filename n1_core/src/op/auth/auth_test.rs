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
    pub owner_id: u32,
    pub app_id: u16,
}

impl DTO for AccessDTO {
    fn check(&self) -> Result<()> {
        if self.owner_id == 0 {
            errs::FIELD_ID_ZERO.push(format!("integer field `owner_id` is zero"))?;
        }
        if self.app_id == 0 {
            errs::FIELD_EMPTY.push(format!("integer field app is zero"))?;
        }
        Ok(())
    }
}

pub async fn auth_test_access_read(
    _server: &OpServer<impl Config>,
    r: OpRequest<AccessDTO>,
) -> Result<Json<bool>> {
    Ok(Json(
        r.token
            .check_access_read(r.dto.owner_id, r.dto.app_id)
            .is_ok(),
    ))
}

pub async fn auth_test_access_write(
    _server: &OpServer<impl Config>,
    r: OpRequest<AccessDTO>,
) -> Result<Json<bool>> {
    Ok(Json(
        r.token
            .check_access_write(r.dto.owner_id, r.dto.app_id)
            .is_ok(),
    ))
}
