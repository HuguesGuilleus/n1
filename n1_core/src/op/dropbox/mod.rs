mod public;
mod text;

use crate::{
    op::{OID_ENTITY_DROPBOX, compo},
    *,
};
use n1_html::{DirectHTML, H, Html};
use n1_tool::{Chunk, Config, chuncks_len};
pub use public::*;
use serde::{Deserialize, Serialize};
pub use text::*;

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq)]
pub struct State {
    // The shadown nb, used to publish page.
    // Zero mean to public page.
    pub shadow: u32,

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

pub async fn page(server: &OpServer<impl Config>, r: OpRequest<u32>) -> Result<String> {
    if r.dto != r.token.uid {
        r.token.check_access_write(r.dto, OID_ENTITY_DROPBOX)?;
    }

    let state: State = server
        .config
        .obj_fetch(r.dto, OID_ENTITY_DROPBOX as u32)
        .await?;
    let entities = server.entities.read().map_err(AtomicError::from)?;
    let owner = entities.get(&r.dto).ok_or(errs::NOT_FOUND_ENTITY)?;

    Ok([H - "html lang=fr"
        + [H - "head" + front::HEAD + [H - "title" + "Dropbox @" + owner.name()]]
        + [H - "body"
            + compo::header( H - "" + "Dropbox @" + owner.name())
            + [H - "main.w"
                + [H - "div.fh.gap"
                    + || {
                        r.token
                        .names(&entities)
                            .map(|(item , name)| H - "a.bl href=/_dropbox/" - item.id +"@"+ name)
                    }]
                + (!state.texts.is_empty()).then(|| [H - "h2" + "Textes"])
                + [H + || {
                    state.texts.iter().map(|(id, text)| {
                        H - "div.bl.mv"
                            + text
                            + [H - "button.bl.mt onclick=textRm(event) data-eid="  -owner.id() - " data-oid=" - *id + "Supprimer"]
                    })
                }]
                + (!state.files.is_empty()).then(|| [H - "h2" + "Fichiers"])
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
            );

            const textRm = ({target}) => {
                fetch("/:dropbox.text.rm", {
                    method: 'PUT',
                    body: JSON.stringify({
                        eid: parseInt(target.dataset.eid),
                        oid: parseInt(target.dataset.oid),
                    })
                }).then(_ => target.parentElement.remove())
            };

        "#)]]
    .render_page())
}
