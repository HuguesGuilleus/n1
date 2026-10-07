use bytes::Bytes;
use serde::{Deserialize, Serialize};

use n1_html::{H, Html, Q};
use n1_tool::{DB, mime};

use crate::{
    DTO, OpRequest, OpServer, Result, errs, front,
    op::{Json, OID_ENTITY_BIO, user::Entity},
};

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct BioState {
    /// The content of the bio.
    pub content: String,
    /// The timestamp of the last edit.
    pub last_edit: u64,
}

pub async fn get(server: &OpServer<impl DB>, r: OpRequest<u32>) -> Result<Json<BioState>> {
    Ok(Json(
        server.config.obj_get(r.dto, OID_ENTITY_BIO as u32).await?,
    ))
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct BioSetRequest {
    oid: u32,
    content: String,
}
impl DTO for BioSetRequest {
    fn check(&self) -> Result<()> {
        if self.oid == 0 {
            errs::FIELD_ID_ZERO.push_result("field `oid`")?;
        }
        Ok(())
    }
}
pub async fn set(server: &OpServer<impl DB>, r: OpRequest<BioSetRequest>) -> Result<()> {
    r.token.check_access_write(r.dto.oid, OID_ENTITY_BIO)?;

    // Check entity
    let entities = server.entities.read().await;
    let entity = entities
        .get(&r.dto.oid)
        .ok_or(errs::NOT_FOUND.push(format!("Cannot get entity with id={}", r.dto.oid)))?;
    if entity.is_none() {
        return errs::EXPECT_REEL_ENTITY.push_result("entity is none");
    }

    // Save the bio state
    let bio = BioState {
        content: r.dto.content,
        last_edit: server.config.now()?,
    };

    server
        .config
        .obj_set(r.dto.oid, OID_ENTITY_BIO as u32, &bio)
        .await?;

    // Generate the bio page
    bio_generate_page(server, entity, &bio).await?;

    Ok(())
}

pub async fn generate_all_pages(server: &OpServer<impl DB>) -> Result<()> {
    let entities = server.entities.read().await;

    for entity in entities.values() {
        if entity.is_none() {
            continue;
        }

        let bio = server
            .config
            .obj_get(entity.id(), OID_ENTITY_BIO as u32)
            .await?;

        bio_generate_page(server, entity, &bio).await?;
    }

    Ok(())
}

async fn bio_generate_page(
    server: &OpServer<impl DB>,
    entity: &Entity,
    bio: &BioState,
) -> Result<()> {
    if entity.is_none() {
        return Ok(());
    }

    let h = [H - "html lang=fr"
        + [H - "head" + front::HEAD + [H - "title" + "@" + entity.name()]]
        + [H - "body"
            + [H - "h1.big" + entity.name()]
            + [H - "main.w"
                + [H - "div.fh.mv.gap"
                    + [H + || {
                        bio.content
                            .split('\n')
                            .map(|line| match line.split_once(' ') {
                                Some((url, text)) if url.starts_with("me+https://") => {
                                    Some([H - "a.bl reel=me href=" - Q(&url[3..]) + text])
                                }
                                _ => None,
                            })
                    }]]
                + [H + || {
                    bio.content
                        .split('\n')
                        .map(|line| match line.split_once(' ') {
                            None if line.starts_with("https://") => {
                                (Some([H - "a.bl.mv href=" - Q(line) + line]), None)
                            }
                            Some((url, _)) if url.starts_with("me+https://") => (None, None),
                            Some((url, text)) if url.starts_with("https://") => {
                                (Some([H - "a.bl.mv href=" - Q(url) + text]), None)
                            }
                            _ => (None, Some([H - "p" + line])),
                        })
                }]
                + ""]
            + [H - "footer" + [H - "a.small href=/ " + "Accueil"]]
            + ""]
        + ""]
    .render_page();

    server
        .config
        .page_set(
            &format!("/@{}/", entity.name()),
            mime::HTML,
            Bytes::from_owner(h),
        )
        .await?;

    Ok(())
}
