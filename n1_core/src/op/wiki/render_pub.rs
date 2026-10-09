use bytes::Bytes;
use n1_html::{DirectHTML, H, Html};
use n1_tool::{DB, Result, mime};

use crate::{
    Common, front,
    op::{
        EntityAndObjectDTO, OID_ENTITY_WIKI, compo,
        user::{Entity, Group},
        wiki::{Article, State},
    },
};

pub async fn render_pub(db: &impl DB, server: &Common) -> Result<()> {
    let entities = server.entities.read().await;
    for owner in entities.values() {
        if let Entity::Group(group) = owner {
            render_pub_one(db, group)
                .await
                .err()
                .inspect(|err| eprintln!("in wiki load of {}: {:?}", owner.id(), err));
        }
    }

    Ok(())
}

async fn render_pub_one(db: &impl DB, owner: &Group) -> Result<()> {
    let state: State = db.obj_get(owner.gid, OID_ENTITY_WIKI as u32).await?;
    if state.shadow == 0 {
        return Ok(());
    }

    render_pub_index(db, owner, &state).await?;
    for page in &state.articles {
        render_pub_page(db, owner, &state, &page).await?;
    }

    Ok(())
}

async fn render_pub_index(db: &impl DB, owner: &Group, state: &State) -> Result<()> {
    let mut pages: Vec<&Article> = state.articles.iter().collect();
    pages.sort_by(|p1, p2| p1.slug.cmp(&p2.slug));
    db.page_set(
        &format!("/.{}/", state.shadow),
        mime::HTML,
        Bytes::from_owner(
            [H - "html lang=fr"
                + [H - "head" + front::HEAD + [H - "title" + "Wiki de " + &owner.name]]
                + [H - "body"
                    + [H - "main.w"
                        + [H - "div.fv.gap"
                            + || {
                                pages.iter().map(|&page| {
                                    [H - "a.bl href="
                                        - format!("/.{}/{}-{}", state.shadow, page.oid, page.slug)
                                        + &page.title
                                        + ""]
                                })
                            }]]
                    + [H - "script" + compo::LOGIN_JS + compo::TIME_JS]
                    + ""]]
            .render_page(),
        ),
    )
    .await?;

    Ok(())
}

async fn render_pub_page(db: &impl DB, owner: &Group, state: &State, page: &Article) -> Result<()> {
    let content = String::from_utf8_lossy(&db.fs_get(state.eid, page.oid).await?).to_string();

    db.page_set(
        &format!("/.{}/{}-{}", state.shadow, page.oid, page.slug),
        mime::HTML,
        Bytes::from_owner(
            [H - "html lang=fr"
                + [H - "head" + front::HEAD + [H - "title" + &page.title]]
                + [H - "body"
                    + compo::header(
                        H + [H - "a.bl href=/@" - &owner.name + "@" + &owner.name]
                            + [H - "a.bl href=./ " + "[wiki]"]
                            + [H - "div.bl"
                                + &page.title
                                + " "
                                + [H - "a.bg href=/_wiki_page/"
                                    - EntityAndObjectDTO {
                                        eid: owner.gid,
                                        oid: page.oid,
                                    }
                                    + "Éditer la page"]
                                + ""],
                    )
                    + compo::header(H - "Wiki" + "@" + &owner.name)
                    + [H - "main.w"
                        + [H - "div"
                            + [H - "i" + "Modification: " + [H - "time" + page.last_edit] + ""]]
                        + [H - "" + || content.lines().map(|line| H - "p" + line)]
                        + ""]
                    + ""]
                + [H - "script" + compo::LOGIN_JS + compo::TIME_JS + DirectHTML(r#""#)]
                + ""]
            .render_page(),
        ),
    )
    .await?;

    Ok(())
}
