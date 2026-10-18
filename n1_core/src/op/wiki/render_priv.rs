use n1_html::{H, Html, Q};
use n1_tool::{DB, Result};

use crate::{
    OpRequest,
    errs::{self},
    front,
    op::{EntityAndObjectDTO, OID_ENTITY_WIKI, compo, wiki::State},
};

pub async fn render_priv_index(db: &impl DB, r: OpRequest<u32>) -> Result<String> {
    r.token.check_access_write(r.dto, OID_ENTITY_WIKI)?;

    let owner_id = r.dto;
    let owner_name = r.common.get_entity_name(owner_id).await?;
    let wiki: State = db.obj_get(r.dto, OID_ENTITY_WIKI as u32).await?;

    Ok([H - "html lang=fr"
        + [H - "head" + front::HEAD + [H - "title" + "Wiki"]]
        + [H - "body"
            + compo::header(
                H + [H - "a.bg href=/_wiki/" - owner_id + "wiki" + " @" + &owner_name]
                    + " "
                    + "Édition du wiki",
            )
            + [H - "main.w.fv.gap"
                + [H - "div.fh" + [H - "a.bl href=/_wiki_new/" - owner_id + "Nouvelle page"]]
                + (|| {
                    wiki.articles.values().map(|article| {
                        H - "a.bl href=/_wiki_page/"
                            - EntityAndObjectDTO {
                                eid: owner_id,
                                oid: article.article_id,
                            }
                            + [H - "div" + [H - "b" + &article.title]]
                            + [H - "div" + ".../" + article.article_id + "-" + &article.slug]
                            + [H - "time" + article.last_edit]
                    })
                })
                + ""]
            + ""]
        + [H - "script" + compo::TIME_JS]
        + [H - ""]
        + ""]
    .render_page())
}

pub async fn render_priv_page(db: &impl DB, r: OpRequest<EntityAndObjectDTO>) -> Result<String> {
    r.token.check_access_write(r.dto.eid, OID_ENTITY_WIKI)?;

    let owner_id = r.dto.eid;
    let owner_name = r.common.get_entity_name(owner_id).await?;
    let wiki: State = db.obj_get(r.dto.eid, OID_ENTITY_WIKI as u32).await?;
    let article = wiki
        .articles
        .get(&r.dto.oid)
        .ok_or_else(|| errs::NOT_FOUND.push(format!("Article {} not found", r.dto.oid)))?;
    let content = db.fs_get(r.dto.eid, r.dto.oid).await?;

    Ok([H - "html lang=fr"
        + [H - "head" + front::HEAD + [H - "title" + "Wiki " + &article.title]]
        + [H - "body"
            + compo::header(H +
                [H - "a.bg href=/_wiki/" - owner_id + "wiki"+ " @" + &owner_name ]
                + " " + "Modification d'une page")
            + [H - "main.w.fv.gap"
                + [H - "i" + "Modification: " + [H - "time" + article.last_edit]]
                + [H - "div.bl.fv.gap"
                    + [H - "div" + [H - "input.bl placeholder=Titre value=" - Q(&article.title)]]
                    + [H - "div" + [H - "button.bl" + "Modifier le titre"]]
                    + ""]
                + [H - "div.bl.fv.gap"
                    + [H - "p"
                        + "Détermine la fin de l'URL de la page. Vous pouver vous inspirer du titre. Ne sont pas acceptés les espaces et les majuscules."]
                    + [H - "div" + [H - "input.bl placeholder=Slug value=" - Q(&article.slug)]]
                    + [H - "div" + [H - "button.bl" + "Modifier le slug"]]]
                + [H - "div.bl.fv.gap"
                    + [H - "div.bl.act contenteditable=plaintext-only"
                        + str::from_utf8(&content).map_err(|_| errs::DB_DECODE)?]
                    + [H - "div" + [H - "button.bl" + "Envoyer"]]
                    + ""]
                + [H - "div" + [H - "button.bl" + "Supprimer"]]
                + ""]
            + ""]
        + [H - "script" + compo::TIME_JS]
        + ""]
    .render_page())
}

pub async fn render_priv_new(_: &impl DB, r: OpRequest<u32>) -> Result<String> {
    r.token.check_access_write(r.dto, OID_ENTITY_WIKI)?;

    let owner_id = r.dto;
    let owner_name = r.common.get_entity_name(r.dto).await?;

    Ok([H - "html lang=fr"
        + [H - "head" + front::HEAD + [H - "title" + "Nouvelle page du wiki"]]
        + [H - "body"
            + compo::header(
                H + [H - "a.bg href=/_wiki/" - owner_id + "wiki" + " @" + &owner_name]
                    + " "
                    + "Nouvelle d'une page",
            )
            + [H - "main.w"
                + [H - "div.bl.fv.gap"
                    + [H - "div" + [H - "input.bl placeholder=Titre"]]
                    + [H - "div" + [H - "input.bl placeholder='Identifiant de l URL'"]]
                    + [H - "div" + [H - "button.bl" + "Nouvelle page"]]
                    + ""]
                + ""]
            + ""]
        + [H - "script" + compo::TIME_JS]
        + ""]
    .render_page())
}
