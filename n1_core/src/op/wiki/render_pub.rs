use bytes::Bytes;
use n1_html::{H, Html};
use n1_tool::{DB, mime};

use crate::{
    Common, Result, front,
    op::{
        Entity, EntityAndObjectDTO, Lang, compo,
        wiki::{WikiArticle, WikiState},
    },
};

/// Render the public wiki articles for all entities.
pub async fn render_pub_all(db: &impl DB, common: &Common) -> Result<()> {
    let entities = common.entities.read().await;
    for owner in entities.values() {
        render_pub_entity(db, common, owner)
            .await
            .err()
            .inspect(|err| eprintln!("in wiki load of {}: {:?}", owner.id(), err));
    }
    Ok(())
}

/// Render public articles for a specific entity's wiki.
async fn render_pub_entity(db: &impl DB, common: &Common, owner: &Entity) -> Result<()> {
    if owner.is_none() {
        return Ok(());
    }

    let guard = common.wiki.get(owner.id()).await?;
    let wiki = guard.read().await;
    if wiki.articles.len() == 0 {
        db.page_del(&format!("/@{}/w/", owner.name())).await?;
    } else {
        render_pub_index(db, common, &owner, &wiki).await?;
        for article in wiki.articles.values() {
            let raw_content = db.fs_get(owner.id(), article.id).await?;
            let content = &String::from_utf8_lossy(&raw_content);
            render_pub_article(db, &common.lang, owner, &article, content).await?;
        }
    }
    Ok(())
}

/// Render the public index article for a specific entity's wiki.
pub async fn render_pub_index(
    db: &impl DB,
    common: &Common,
    owner: &Entity,
    state: &WikiState,
) -> Result<()> {
    let mut articles = state.articles.values().collect::<Vec<_>>();
    articles.sort_by_key(|article| &article.slug);

    let lang = common.lang;
    db.page_set(
        &format!("/@{}/w/", owner.name()),
        mime::HTML,
        Bytes::from_owner(
            [H - "html lang=fr"
                + [H - "head" + front::HEAD + [H - "title" + lang.wiki + " @" + owner.name()]]
                + [H - "body"
                    + compo::goto_main(lang)
                    + [H - "header.fh.gap"
                        + [H - "a.bl href=/@" - owner.name() - "/ " + "@" + owner.name()]
                        + [H - "div.bl" + lang.wiki]
                        + [H - "a.bl.needAuth href=/_wiki/" - owner.id() + lang.common_edit]
                        + compo::header_menu(lang)]
                    + [H - "main.w id=main"
                        + [H - "div.fv.gap"
                            + || {
                                articles.iter().map(|&article| {
                                    [H - "a.bl href="
                                        - format!(
                                            "/@{}/w/{}-{}",
                                            owner.name(),
                                            article.id,
                                            article.slug
                                        )
                                        + &article.title
                                        + ""]
                                })
                            }]]
                    + [H - "footer"
                        + [H - "div"
                            + lang.common_lastedit
                            + [H - "time"
                                + articles
                                    .iter()
                                    .fold(0u64, |acc, article| acc.max(article.last_edit))
                                + ""]]
                        + compo::footer_home(lang)]
                    + ""]
                + [H - "script" + compo::LOGIN_JS + compo::TIME_JS]
                + ""]
            .render_page(),
        ),
    )
    .await?;

    Ok(())
}

/// Public render of one article.
pub async fn render_pub_article(
    db: &impl DB,
    lang: &Lang,
    owner: &Entity,
    article: &WikiArticle,
    content: &str,
) -> Result<()> {
    db.page_set(
        &format!("/@{}/w/{}-{}", owner.name(), article.id, article.slug),
        mime::HTML,
        Bytes::from_owner(
            [H - "html lang=fr"
                + [H - "head" + front::HEAD + [H - "title" + &article.title]]
                + [H - "body"
                    + compo::goto_main(lang)
                    + [H - "header.fh.gap"
                        + [H - "a.bl href=/@" - owner.name() - "/ " + "@" + owner.name()]
                        + [H - "a.bl href=./ " + lang.wiki]
                        + [H - "div.bl" + article.id + "-" + &article.slug + ""]
                        + [H - "a.bl.needAuth id=edit href=/_wiki_article/"
                            - EntityAndObjectDTO {
                                owner: owner.id(),
                                object: article.id,
                            }
                            + lang.common_edit
                            + ""]
                        + compo::header_menu(lang)
                        + ""]
                    + [H - "main.w id=main"
                        + [H - "h1.bl" + &article.title + ""]
                        + [|| content.lines().map(|line| H - "p" + line)]
                        + ""]
                    + ""]
                + [H - "footer"
                    + [H - "div" + lang.common_lastedit + [H - "time" + article.last_edit]]
                    + compo::footer_home(lang)
                    + ""]
                + [H - "script" + compo::LOGIN_JS + compo::TIME_JS]
                + ""]
            .render_page(),
        ),
    )
    .await?;
    Ok(())
}
