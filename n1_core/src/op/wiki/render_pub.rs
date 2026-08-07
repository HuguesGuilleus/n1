use bytes::Bytes;
use n1_html::{DirectHTML, H, Html};
use n1_tool::{Config, Result, mime};

use crate::{
    OpServer, front,
    op::{
        EntityAndObjectDTO, OID_ENTITY_WIKI, compo,
        user::{Entity, Group},
        wiki::{Page, State},
    },
};

pub async fn render_pub<C: Config>(server: &OpServer<C>) -> Result<()> {
    let entities = server.entities.read()?;
    for owner in entities.values() {
        if let Entity::Group(group) = owner {
            render_pub_one(server, group)
                .await
                .err()
                .inspect(|err| eprintln!("in wiki load of {}: {:?}", owner.id(), err));
        }
    }

    Ok(())
}

async fn render_pub_one<C: Config>(server: &OpServer<C>, owner: &Group) -> Result<()> {
    let state: State = server.config.obj_fetch(owner.gid, OID_ENTITY_WIKI).await?;
    if state.shadow == 0 {
        return Ok(());
    }

    render_pub_index(server, owner, &state).await?;
    for page in &state.pages {
        render_pub_page(server, owner, &state, &page).await?;
    }

    Ok(())
}

async fn render_pub_index<C: Config>(
    server: &OpServer<C>,
    owner: &Group,
    state: &State,
) -> Result<()> {
    let mut pages: Vec<&Page> = state.pages.iter().collect();
    pages.sort_by(|p1, p2| p1.slug.cmp(&p2.slug));
    server
        .config
        .page_add(
            &format!("/.{}/", state.shadow),
            mime::HTML,
            Bytes::from_owner(
                [H - "html lang=fr"
                    + [H - "head" + front::HEAD + [H - "title" + "Wiki de " + &owner.name]]
                    + [H - "body"
                        + compo::header2(
                            H + [H - "a.bl href=/@" - &owner.name + "@" + &owner.name]
                                + [H - "a.bl href=./ " + "[wiki]"],
                        )
                        + [H - "main.w"
                            + [H - "div.fv.gap"
                                + || {
                                    pages.iter().map(|&page| {
                                        [H - "a.bl href="
                                            - format!(
                                                "/.{}/{}-{}",
                                                state.shadow, page.id, page.slug
                                            )
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

async fn render_pub_page<C: Config>(
    server: &OpServer<C>,
    owner: &Group,
    state: &State,
    page: &Page,
) -> Result<()> {
    let content =
        String::from_utf8_lossy(&server.config.fs_get(state.eid, page.id).await?).to_string();

    server
        .config
        .page_add(
            &format!("/.{}/{}-{}", state.shadow, page.id, page.slug),
            mime::HTML,
            Bytes::from_owner(
                [H - "html lang=fr"
                    + [H - "head" + front::HEAD + [H - "title" + &page.title]]
                    + [H - "body"
                        + compo::header2(
                            H + [H - "a.bl href=/@" - &owner.name + "@" + &owner.name]
                                + [H - "a.bl href=./ " + "[wiki]"]
                                + [H - "div.bl"
                                    + &page.title
                                    + " "
                                    + [H - "a.bg href=/_wiki_page/"
                                        - EntityAndObjectDTO {
                                            eid: owner.gid,
                                            oid: page.id,
                                        }
                                        + "Éditer la page"]
                                    + ""],
                        )
                        + [H - "main.w"
                            + [H - "div"
                                + [H - "i"
                                    + "Modification: "
                                    + [H - "time" + page.last_edit]
                                    + ""]]
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
