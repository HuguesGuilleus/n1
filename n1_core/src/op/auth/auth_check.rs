use n1_tool::Config;

use crate::op::AccessDTO;
use crate::{OpRequest, OpServer, Result};

pub async fn auth_check_auth(_server: &OpServer<impl Config>, r: OpRequest<()>) -> Result<()> {
    r.token.check_auth()?;
    Ok(())
}

pub async fn auth_check_isadmin(_server: &OpServer<impl Config>, r: OpRequest<()>) -> Result<()> {
    r.token.check_isadmin()?;
    Ok(())
}

pub async fn auth_check_access_read(
    _server: &OpServer<impl Config>,
    r: OpRequest<AccessDTO>,
) -> Result<()> {
    r.token.check_access_read(r.dto.owner_id, r.dto.app_id)
}

pub async fn auth_check_access_write(
    _server: &OpServer<impl Config>,
    r: OpRequest<AccessDTO>,
) -> Result<()> {
    r.token.check_access_write(r.dto.owner_id, r.dto.app_id)
}
