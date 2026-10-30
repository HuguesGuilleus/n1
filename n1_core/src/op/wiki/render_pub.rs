use bytes::Bytes;
use n1_html::{DirectHTML, H, Html};
use n1_tool::{DB, Result, mime};

use crate::{
    Common, front,
    op::{
        Entity, EntityAndObjectDTO, OID_ENTITY_WIKI, compo,
        wiki::{Article, State},
    },
};

/// Render the public wiki pages for groups and users.
pub async fn render_pub(db: &impl DB, server: &Common) -> Result<()> {
    let entities = server.entities.read().await;
    for owner in entities.values() {
        render_pub_for_entity(db, owner)
            .await
            .err()
            .inspect(|err| eprintln!("in wiki load of {}: {:?}", owner.id(), err));
    }

    Ok(())
}

async fn render_pub_for_entity(db: &impl DB, owner: &Entity) -> Result<()> {
    if owner.is_none() {
        return Ok(());
    }

    let state: State = db.obj_get(owner.id(), OID_ENTITY_WIKI as u32).await?;
    if state.owner_id == 0 {
        return Ok(());
    }

    render_one_wiki(db, owner.name(), &state).await?;

    Ok(())
}

pub async fn render_one_wiki(db: &impl DB, owner_name: &str, wiki: &State) -> Result<()> {
    render_pub_index(db, owner_name, wiki).await?;
    for article in wiki.articles.values() {
        render_pub_page(db, owner_name, wiki, article).await?;
    }
    Ok(())
}

async fn render_pub_index(db: &impl DB, owner_name: &str, state: &State) -> Result<()> {
    let mut articles = state.articles.values().collect::<Vec<_>>();
    articles.sort_by_key(|article| &article.slug);

    db.page_set(
        &format!("/@{}/w/", &owner_name),
        mime::HTML,
        Bytes::from_owner(
            [H - "html lang=fr"
                + [H - "head" + front::HEAD + [H - "title" + "Wiki de " + owner_name]]
                + [H - "body"
                    + [H - "header.fh.gap"
                        + [H - "div.bl.fh.gap"
                            + [H - "a.bg href=/@" - owner_name + "@" + owner_name]
                            + "[wiki]"
                            + ""]
                        + [H - "a.mlauto.bl href=/_" + "Menu"]
                        + ""]
                    + [H - "main.w"
                        + [H - "div.fv.gap"
                            + || {
                                articles.iter().map(|&article| {
                                    [H - "a.bl href="
                                        - format!(
                                            "/@{}/w/{}-{}",
                                            owner_name, article.article_id, article.slug
                                        )
                                        + &article.title
                                        + ""]
                                })
                            }]]
                    + [H - "script" + compo::LOGIN_JS + compo::TIME_JS]
                    + ""]
                + ""]
            .render_page(),
        ),
    )
    .await?;

    Ok(())
}

async fn render_pub_page(
    db: &impl DB,
    owner_name: &str,
    wiki: &State,
    article: &Article,
) -> Result<()> {
    let content =
        String::from_utf8_lossy(&db.fs_get(wiki.owner_id, article.article_id).await?).to_string();

    db.page_set(
        &format!("/@{}/w/{}-{}", owner_name, article.article_id, article.slug),
        mime::HTML,
        Bytes::from_owner(
            [H - "html lang=fr"
                + [H - "head" + front::HEAD + [H - "title" + &article.title]]
                + [H - "body"
                    + [H - "header.fh.gap"
                        + [H - "div.bl.fh.gap"
                            + [H - "a.bg href=/@" - owner_name + "@" + owner_name]
                            + [H - "a.bg href=./ " + "[wiki]"]
                            + [H - "h1" + &article.title + ""]
                            + [H - "a.bg href=/_wiki_page/"
                                - EntityAndObjectDTO {
                                    owner: wiki.owner_id,
                                    object: article.article_id,
                                }
                                + "Éditer la page"]
                            + ""]
                        + [H - "a.mlauto.bl href=/_" + "Menu"]
                        + ""]
                    + [H - "main.w"
                        + [H - "div"
                            + [H - "i"
                                + "Dernière modification\u{202F}: "
                                + [H - "time" + article.last_edit]
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
