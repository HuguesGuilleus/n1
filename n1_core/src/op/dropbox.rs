use crate::{
    op::{OID_ENTITY_DROPBOX, compo},
    *,
};
use n1_html::{DirectHTML, H, Html};
use n1_tool::{Chunk, Config, chuncks_len};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq)]
pub struct State {
    pub texts_inc: u32,
    pub texts: Vec<(u32, String)>,

    pub files_inc: u32,
    pub files: Vec<DropFile>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq)]
pub struct DropFile {
    pub id: u32,
    pub name: String,
    pub upload_secs: u64,
    pub chunks: Vec<Chunk>,
}

pub async fn page(server: &OpServer<impl Config>, r: OpRequest<ID>) -> Result<String> {
    if r.dto.0 != r.token.uid {
        r.token.group_global(r.dto.0, op::TokenLevel::Write)?;
    }

    let state: State = server.config.obj_fetch(r.dto.0, OID_ENTITY_DROPBOX).await?;
    let entities = server.entities.read()?;
    let owner = entities.get(&r.dto.0).ok_or(errs::NOT_FOUND_ENTITY)?;

    Ok([H - "html lang=fr"
        + [H - "head" + front::HEAD + [H - "title" + "Dropbox @" + owner.name()]]
        + [H - "body"
            + compo::header(true, H - "" + "Dropbox @" + owner.name())
            + [H - "main.w"
                + [H - "div.fh.gap"
                    + || {
                        r.token
                            .groups()
                            .flat_map(|(gid, _)| {
                                entities.get(&gid).map(|e| (gid, e.name()))
                            })
                            .map(|(gid, name)| H - "a.bl href=/_dropbox/" - gid + name)
                    }]
                + (!state.texts.is_empty()).then(|| [H - "h1" + "Textes"])
                + [H + || {
                    state.texts.iter().map(|(id, text)| {
                        H - "div.bl.mv"
                            + text
                            + [H - "button.bl.mt data-id=" - *id + "Supprimer"]
                    })
                }]
                + (!state.files.is_empty()).then(|| [H - "h1" + "Fichiers"])
                + [H + || {
                    state.files.iter().map(|f| {
                        H - "div.bl.mv"
                            + [H - "b" + &f.name]
                            + [H - "div"
                                + "("
                                + chuncks_len(&f.chunks)
                                + "octet) "
                                + [H - "time" + f.upload_secs]]
                            + [H - "div.fh.gap"
                                + [H - "a.bl.mt data-id=" - f.id + "Télécharger"]
                                + [H - "button.bl.mt data-id=" - f.id + "Supprimer"]
                                + ""]
                    })
                }]
                + ""]
            + ""]
        + [H - "script" + DirectHTML(r#"document.querySelectorAll("time").forEach(
            t=>t.innerText = new Intl.DateTimeFormat(document.documentElement.lang,{dateStyle:"full",timeStyle:"long"})
                .format(new Date(parseInt(t.innerText)*1000))
            );"#)]]
    .render_page())
}
