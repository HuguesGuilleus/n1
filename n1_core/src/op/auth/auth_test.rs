use n1_tool::DB;
use serde::Deserialize;

use crate::{DTO, OpRequest, OpServer, Result, errs};

pub async fn auth_test_id(_server: &OpServer<impl DB>, r: OpRequest<()>) -> Result<u32> {
    Ok(r.token.uid)
}

pub async fn auth_test_isadmin(_server: &OpServer<impl DB>, r: OpRequest<()>) -> Result<bool> {
    Ok(r.token.is_admin)
}

#[derive(Debug, Deserialize)]
pub struct AccessDTO {
    pub owner_id: u32,
    pub app_id: u16,
}

impl DTO for AccessDTO {
    fn check(&self) -> Result<()> {
        if self.owner_id == 0 {
            errs::FIELD_ID_ZERO.push_result(format!("integer field `owner_id` is zero"))?;
        }
        if self.app_id == 0 {
            errs::FIELD_EMPTY.push_result(format!("integer field app is zero"))?;
        }
        Ok(())
    }
}

pub async fn auth_test_access_read(
    _server: &OpServer<impl DB>,
    r: OpRequest<AccessDTO>,
) -> Result<bool> {
    Ok(r.token
        .check_access_read(r.dto.owner_id, r.dto.app_id)
        .is_ok())
}

pub async fn auth_test_access_write(
    _server: &OpServer<impl DB>,
    r: OpRequest<AccessDTO>,
) -> Result<bool> {
    Ok(r.token
        .check_access_write(r.dto.owner_id, r.dto.app_id)
        .is_ok())
}
