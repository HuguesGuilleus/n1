use bytes::Bytes;
use serde::{Deserialize, Serialize};

use n1_html::{DirectHTML, H, Html, Q};
use n1_tool::{DB, mime};

use crate::{Common, DTO, OpRequest, Result, errs, front, op::OID_ENTITY_BIO};

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct BioState {
    /// The content of the bio.
    pub content: String,
    /// The timestamp of the last edit.
    pub last_edit: u64,
}

pub async fn get(db: &impl DB, r: OpRequest<u32>) -> Result<BioState> {
    Ok(db.obj_get(r.dto, OID_ENTITY_BIO as u32).await?)
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
pub async fn set(db: &impl DB, r: OpRequest<BioSetRequest>) -> Result<()> {
    r.token.check_access_write(r.dto.oid, OID_ENTITY_BIO)?;

    // Check entity
    let entity_name = r.common.get_entity_name(r.dto.oid).await?;

    // Save the bio state
    let bio = BioState {
        content: r.dto.content,
        last_edit: db.now()?,
    };

    db.obj_set(r.dto.oid, OID_ENTITY_BIO as u32, &bio).await?;

    // Generate the bio page
    generate_one_page(db, &entity_name, &bio).await?;

    Ok(())
}

pub async fn generate_all_pages(db: &impl DB, common: &Common) -> Result<()> {
    let entities = common.entities.read().await;

    for entity in entities.values() {
        if entity.is_none() {
            continue;
        }

        let bio = db.obj_get(entity.id(), OID_ENTITY_BIO as u32).await?;

        generate_one_page(db, entity.name(), &bio).await?;
    }

    Ok(())
}

async fn generate_one_page(db: &impl DB, entity_name: &str, bio: &BioState) -> Result<()> {
    let path = format!("/@{}/", entity_name);
    if bio.content.is_empty() {
        db.page_set(&path, "", Bytes::new()).await?;
        return Ok(());
    }

    let h = [H - "html lang=fr"
        + [H - "head" + front::HEAD + [H - "title" + "@" + entity_name]]
        + [H - "body"
            + [H - "h1.big" + "@" + entity_name]
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

    db.page_set(&path, mime::HTML, Bytes::from_owner(h)).await?;

    Ok(())
}

pub async fn console(db: &impl DB, r: OpRequest<u32>) -> Result<String> {
    r.token.check_access_write(r.dto, OID_ENTITY_BIO)?;
    let entity_name = r.common.get_entity_name(r.dto).await?;
    let bio: BioState = db.obj_get(r.dto, OID_ENTITY_BIO as u32).await?;

    let h = [H - "html lang=fr"
        + [H - "head" + front::HEAD + [H - "title" + "!Modifie la page de @" + &entity_name]]
        + [H - "body"
            + [H - "header" + [H - "h1.bl" + "!Modifie la page de @" + &entity_name]]
            + [H - "main.w"
                + [H - "div.fh.mv"
                    + [H - "a.bl href=/@" - &entity_name - "/" + "@" + &entity_name]]
                + [H - "div.bb.act"
                    + [H - "p"
                        + "Chaque ligne est analysée séparément: comme un lien (débutant par l'URL), un lien de réseau sociaux (débutant par 'me+https://'), ou simplement un paragraphe."]
                    + [H - "div hidden id=_oid" + r.dto]
                    + [H - "pre.bl.act id=_content contenteditable" + bio.content]
                    + [H - "button.bl.act onclick=send()" + "!Enregistrer"]
                    + ""]
                + ""]
            + [H - "footer" + [H - "a.small href=/ " + "Accueil"]]
            + [H - "script"
                + DirectHTML(
                    r#"const send=()=>{
                        fetch("/:bio.set/", {
                            method: "PUT",
                            body: JSON.stringify({
                                oid: parseInt(_oid.innerText) ,
                                content: _content.innerText ,
                            }),
                        });
                    }"#,
                )]
            + ""]];

    Ok(h.render_page())
}
