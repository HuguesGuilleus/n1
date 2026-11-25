use n1_tool::DB;
use serde::Deserialize;

use crate::{ErrorStep, OpRequest, Result, DTO};

#[derive(Debug, Deserialize)]
pub struct AccessDTO {
    pub owner_id: u32,
    pub app_id: u16,
}

impl DTO for AccessDTO {
    fn check(&self) -> Result<()> {
        if self.owner_id == 0 {
            ErrorStep::FieldZero { name: "owner_id" }.result()?;
        } else if self.app_id == 0 {
            ErrorStep::FieldZero { name: "app_id" }.result()?;
        }
        Ok(())
    }
}

pub async fn auth_check_auth(_: &impl DB, r: OpRequest<()>) -> Result<()> {
    r.token.check_auth()
}

pub async fn auth_check_isadmin(_: &impl DB, r: OpRequest<()>) -> Result<()> {
    r.token.check_isadmin()
}

pub async fn auth_check_access_read(_: &impl DB, r: OpRequest<AccessDTO>) -> Result<()> {
    r.token.check_access_read(r.dto.owner_id, r.dto.app_id)
}

pub async fn auth_check_access_write(_: &impl DB, r: OpRequest<AccessDTO>) -> Result<()> {
    r.token.check_access_write(r.dto.owner_id, r.dto.app_id)
}

pub async fn auth_get_id(_: &impl DB, r: OpRequest<()>) -> Result<u32> {
    Ok(r.token.uid)
}
