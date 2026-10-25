//! A presentation page of the entity (group or user).
//!
//! HURL tst file: <https://github.com/HuguesGuilleus/n1/blob/main/n1_core/tests/bio.hurl>

use bytes::Bytes;
use n1_js::js;
use serde::{Deserialize, Serialize};

use n1_html::{DirectHTML, H, Html, Q};
use n1_tool::{DB, mime};

use crate::{
    Common, DTO, OpRequest, Result, errs, front,
    op::{
        OID_ENTITY_BIO,
        compo::{self, FETCHER_JS, LOGIN_JS, TIME_JS},
    },
};

/// Information stored for on entity.
#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct BioState {
    /// The content of the bio.
    pub content: String,
    /// The timestamp of the last edit.
    pub last_edit: u64,
}

/// JSON API to get information about a entity's bio.
/// Path: `/:bio.get/`
pub async fn get(db: &impl DB, r: OpRequest<u32>) -> Result<BioState> {
    Ok(db.obj_get(r.dto, OID_ENTITY_BIO as u32).await?)
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct BioSetDTO {
    pub owner: u32,
    pub content: String,
}
impl DTO for BioSetDTO {
    fn check(&self) -> Result<()> {
        if self.owner == 0 {
            errs::FIELD_ID_ZERO.push_result("field `oid`")?;
        }
        Ok(())
    }
}
/// JSON API to set information about a entity's bio.
/// Path: `/:bio.set/`
pub async fn set(db: &impl DB, r: OpRequest<BioSetDTO>) -> Result<()> {
    r.token.check_access_write(r.dto.owner, OID_ENTITY_BIO)?;

    // Check entity
    let entity_name = r.common.get_entity_name(r.dto.owner).await?;

    // Save the bio state
    let bio = BioState {
        content: r.dto.content,
        last_edit: db.now()?,
    };
    db.obj_set(r.dto.owner, OID_ENTITY_BIO as u32, &bio).await?;

    // Generate the bio page
    generate_one_page(db, &r.common, &entity_name, &bio).await?;

    Ok(())
}

/// Generate page for all entities.
/// Use it at launch or when common headline or footer change.
pub async fn generate_all_pages(db: &impl DB, common: &Common) -> Result<()> {
    let entities = common.entities.read().await;
    for entity in entities.values() {
        if entity.is_none() {
            continue;
        }
        let bio = db.obj_get(entity.id(), OID_ENTITY_BIO as u32).await?;
        generate_one_page(db, common, entity.name(), &bio).await?;
    }
    Ok(())
}

/// Generate the bio page for a single entity.
/// Path: `/@{entity_name}/`
async fn generate_one_page(
    db: &impl DB,
    common: &Common,
    entity_name: &str,
    bio: &BioState,
) -> Result<()> {
    let path = format!("/@{}/", entity_name);
    if bio.content.is_empty() {
        db.page_set(&path, "", Bytes::new()).await?;
        return Ok(());
    }

    let lang = common.lang;
    let h = [H - "html lang=" - lang.code
        + [H - "head" + front::HEAD + [H - "title" + "@" + entity_name]]
        + [H - "body"
            + compo::goto_main(lang)
            + [H - "header.fh"
                + [H - "h1.bl" + "@" + entity_name]
                + compo::header_menu(&lang)
                + ""]
            + [H - "main.w id=main"
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
            + [H - "footer"
                + [H - "div" + lang.common_lastedit + [H - "time" + bio.last_edit]]
                + compo::footer_home(lang)
                + ""]
            + ""]
        + [H - "script" + compo::LOGIN_JS + compo::TIME_JS]
        + ""]
    .render_page();

    db.page_set(&path, mime::HTML, Bytes::from_owner(h)).await?;

    Ok(())
}

/// Page to edit a bio.
/// Path: `/_bio/{entity_id}`
pub async fn console(db: &impl DB, r: OpRequest<u32>) -> Result<String> {
    r.token.check_access_write(r.dto, OID_ENTITY_BIO)?;

    let entity_name = r.common.get_entity_name(r.dto).await?;
    let bio: BioState = db.obj_get(r.dto, OID_ENTITY_BIO as u32).await?;

    let lang = r.common.lang;
    let h = [H - "html lang=" - lang.code
        + [H - "head" + front::HEAD + [H - "title" + lang.bio_edit_page + &entity_name]]
        + [H - "body"
            + compo::goto_main(lang)
            + [H - "header.fh"
                + [H - "h1.bl" + lang.bio_edit_page + &entity_name]
                + compo::header_menu(lang)
                + ""]
            + [H - "main.w id=main"
                + [H + || {
                    lang.bio_edit_format_info
                        .iter()
                        .map(|&line| [H - "p.nospace" + line])
                }]
                + [H - "pre.bl.act id=content contenteditable" + bio.content]
                + [H - "div.fh.mv.gap"
                    + [H - "button.bl.act onclick=send()"
                        + lang.common_save
                        + " "
                        + [H - "span id=out"]]
                    + [H - "a.bl href=/@" - &entity_name - "/ " + "@" + &entity_name]
                    + ""]
                + ""]
            + [H - "footer"
                + [H - "p.nospace" + lang.common_lastedit + [H - "time" + bio.last_edit]]
                + [H - "a.small href=/ " + "Accueil"]
                + ""]
            + [H - "script"
                + LOGIN_JS
                + TIME_JS
                + [H + "const owner=" + r.dto + ","]
                + [H + FETCHER_JS + ","]
                + DirectHTML(js!(
                    send = () => fetcher(fetch("/:bio.set/", {
                        method: "PUT",
                        body: JSON.stringify({
                            owner,
                            content: content.innerText,
                        }),
                    }), out);
                ))
                + ""]
            + ""]];

    Ok(h.render_page())
}
