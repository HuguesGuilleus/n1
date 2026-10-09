use n1_tool::DB;

use crate::op::AccessDTO;
use crate::{OpRequest, Result};

pub async fn auth_check_auth(_: &impl DB, r: OpRequest<()>) -> Result<()> {
    r.token.check_auth()?;
    Ok(())
}

pub async fn auth_check_isadmin(_: &impl DB, r: OpRequest<()>) -> Result<()> {
    r.token.check_isadmin()?;
    Ok(())
}

pub async fn auth_check_access_read(_: &impl DB, r: OpRequest<AccessDTO>) -> Result<()> {
    r.token.check_access_read(r.dto.owner_id, r.dto.app_id)
}

pub async fn auth_check_access_write(_: &impl DB, r: OpRequest<AccessDTO>) -> Result<()> {
    r.token.check_access_write(r.dto.owner_id, r.dto.app_id)
}
