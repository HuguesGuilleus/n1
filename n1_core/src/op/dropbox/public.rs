use bytes::Bytes;
use n1_html::{DirectHTML, H, Html};
use n1_tool::{DB, mime};

use crate::{
    Common, Result, front,
    op::{OID_ENTITY_DROPBOX, compo, dropbox::State},
};

pub async fn render_public(db: &impl DB, common: &Common) -> Result<()> {
    let entities = common.entities.read().await;
    for (&eid, entity) in entities.iter() {
        let state: State = db.obj_get(eid, OID_ENTITY_DROPBOX as u32).await?;
        if state.shadow == 0 {
            continue;
        }
        db.page_set(
            &format!("/.{}", state.shadow),
            mime::HTML,
            Bytes::from_owner(render_public_one(eid, entity.name())),
        )
        .await?;
    }

    Ok(())
}

pub fn render_public_one(uid: u32, name: &str) -> String {
    [H - "html lang=fr"
        + [H - "head" + front::HEAD + [H - "title" + "Dépôt pour @" + name]]
        + [H - "body"
            + compo::header(H - "" + "Dépôt pour @" + name)
            + [H - "main.w"
                + [H - "h2" + "Envoyer un texte"]
                + [H - "div.bl"
                    + [H - "div.mv id=textData contenteditable=plaintext-only" + "Message"]
                    + [H - "button.bl id=textSend" + "Envoyer"]
                    + ""]
                + [H - "h2" + "Envoyer un fichier"]
                + ""]
            + ""]
        + [H - "script"
            + "const eid="
            + uid
            + ";"
            + DirectHTML(
                r#"
                textSend.onclick = _ => fetch("/:dropbox.text.add", {
                    method: 'PUT',
                    body: JSON.stringify({
                        eid,
                        str: textData.innerText,
                    })
                }).then(r=>textData.innerText='')
                "#,
            )]
        + ""]
    .render_page()
}
