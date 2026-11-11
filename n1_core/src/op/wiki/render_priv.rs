use n1_html::{DirectHTML, H, Html, Q};
use n1_js::js;
use n1_tool::DB;

use crate::{
    ErrorStep, OpRequest, Result, front,
    op::{EntityAndObjectDTO, OID_ENTITY_WIKI, compo},
};

/// The console to edit the wiki articles of the owner ID.
pub async fn render_priv_index(_: &impl DB, r: OpRequest<u32>) -> Result<String> {
    r.token.check_access_write(r.dto, OID_ENTITY_WIKI)?;

    let owner = r.common.get_entity(r.dto).await?;
    let guard = r.common.wiki.get(r.dto).await?;
    let wiki = guard.read().await;

    let lang = r.common.lang;
    Ok([H - "html lang=" - lang.code
        + [H - "head" + front::HEAD + [H - "title" + "@" + owner.name() + " " + lang.wiki]]
        + [H - "body"
            + compo::goto_main(lang)
            + [H - "header.fh.gap"
                + [H - "div.bl" + "@" + owner.name() + " " + lang.wiki]
                + compo::header_menu_logged(lang)]
            + [H - "main.w.fv.gap"
                + [H - "div.fh.gap"
                    + [H - "a.bl href=/_wiki_new/" - owner.id() + lang.wiki_new_article_button]
                    + [H - "a.bl href=/@" - owner.name() - "/w/ " + lang.wiki_public_zone]]
                + (|| {
                    wiki.articles.values().map(|article| {
                        H - "a.bl href=/_wiki_article/"
                            - EntityAndObjectDTO {
                                owner: owner.id(),
                                object: article.id,
                            }
                            + [H - "div" + [H - "b" + &article.title]]
                            + [H - "div" + ".../" + article.id + "-" + &article.slug]
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

/// The console to edit a specific wiki article.
pub async fn render_priv_page(db: &impl DB, r: OpRequest<EntityAndObjectDTO>) -> Result<String> {
    r.token.check_access_write(r.dto.owner, OID_ENTITY_WIKI)?;

    let owner = r.common.get_entity(r.dto.owner).await?;
    let guard = r.common.wiki.get(owner.id()).await?;
    let wiki = guard.read().await;
    let article = wiki
        .articles
        .get(&r.dto.object)
        .ok_or_else(|| ErrorStep::NotFoundObject {
            owner: owner.id(),
            object: r.dto.object,
            type_name: "wiki-page",
        })?;
    let raw_content = db.fs_get(r.dto.owner, r.dto.object).await?;
    let content = String::from_utf8_lossy(&raw_content);

    let lang = r.common.lang;
    Ok([H - "html lang=" - lang.code
        + [H - "head" + front::HEAD + [H - "title" + "Wiki " + &article.title]]
        + [H - "body"
            + compo::goto_main(lang)
            + [H - "header.fh.gap"
                + [H - "div.bl" + "@" + owner.name()]
                + [H - "a.bl href=/_wiki/" - owner.id() + lang.wiki]
                + [H - "div.bl" + lang.wiki_edit_article]
                + compo::header_menu_logged(lang)]
            + [H - "main.w.fv.gap"
                + [H - "div.fh"
                    + [H - "a.bl href=/@"
                        - owner.name()
                        - "/w/"
                        - article.id
                        - "-"
                        - &article.slug
                        + lang.wiki_public_anchor]]
                + [H - "i" + lang.common_lastedit + [H - "time" + article.last_edit]]
                + [H - "div.bb.act.fv.gap"
                    + [H - "div.fv"
                        + [H - "div"
                            + [H - "label for=title" + lang.home_edit_title]
                            + [H - "input.bl id=title value=" - Q(&article.title)]
                            + ""]]
                    + [H - "div" + [H - "button.bl onclick=setTitle()" + lang.common_save]]
                    + [H - "div id=outTitle"]
                    + ""]
                + [H - "div.bb.act.fv.gap"
                    + [H - "label for=slug" + lang.wiki_edit_article_slug]
                    + [H - "div" + [H - "input.bl id=slug value=" - Q(&article.slug)]]
                    + [H - "div" + [H - "button.bl onclick=setSlug()" + lang.common_save]]
                    + [H - "div id=outSlug"]
                    + ""]
                + [H - "div.bb.act.fv.gap"
                    + [H - "label for=content" + lang.wiki_edit_article_content]
                    + [H - "textarea.bl.act id=content" + &*content]
                    + [H - "div" + [H - "button.bl onclick=setContent()" + lang.common_save]]
                    + [H - "div id=outContent"]
                    + ""]
                + [H - "div" + [H - "button.bl onclick=remove()" + lang.common_remove]]
                + [H - "div id=outRemove"]
                + ""]
            + ""]
        + [H - "script"
            + compo::TIME_JS
            + [H + "const owner=" + owner.id() + ",article=" + r.dto.object + ","]
            + [H + compo::fetcher_js(lang) + ","]
            + [H + "RMM=" + Q(lang.common_remove_confirm) + ","]
            + DirectHTML(js!(
                setTitle = _ => F(fetch("/:wiki.article.setTitle/", {
                    method: "PUT",
                    body: JSON.stringify({
                        owner,
                        article,
                        title: title.value,
                    }),
                }), outTitle),
                setSlug = _ => F(fetch("/:wiki.article.setSlug/", {
                    method: "PUT",
                    body: JSON.stringify({
                        owner,
                        article,
                        slug: slug.value,
                    }),
                }), outSlug),
                setContent = _ => F(fetch("/:wiki.article.setContent/", {
                    method: "PUT",
                    body: JSON.stringify({
                        owner,
                        article,
                        content: content.value,
                    }),
                }), outContent),
                remove = _ => confirm(RMM) && F(fetch("/:wiki.article.rm/", {
                    method: "PUT",
                    body: JSON.stringify({ owner, article }),
                }), outRemove).then(_ => location = "/_wiki/" + owner);
            ))]
        + ""]
    .render_page())
}

/// The console to create a new wiki article.
pub async fn render_priv_new(_: &impl DB, r: OpRequest<u32>) -> Result<String> {
    r.token.check_access_write(r.dto, OID_ENTITY_WIKI)?;

    let owner = r.common.get_entity(r.dto).await?;

    let lang = r.common.lang;
    Ok([H - "html lang=" - lang.code
        + [H - "head" + front::HEAD + [H - "title" + lang.wiki_new_article]]
        + [H - "body"
            + compo::goto_main(lang)
            + [H - "header.fh.gap"
                + [H - "div.bl" + "@" + owner.name()]
                + [H - "a.bl href=/_wiki/" - owner.id() + lang.wiki]
                + [H - "div.bl" + lang.wiki_new_article]
                + compo::header_menu_logged(lang)]
            + [H - "main.w id=main"
                + [H - "form.fv.gap.bb.act id=form"
                    + [H - "div"
                        + [H - "label for=title" + lang.wiki_edit_article_title]
                        + [H - "div" + [H - "input.bl id=title required"]]
                        + ""]
                    + [H - "div"
                        + [H - "label for=slug" + lang.wiki_edit_article_slug]
                        + [H - "div" + [H - "input.bl id=slug required"]]
                        + ""]
                    + [H - "div"
                        + [H - "label for=content" + lang.wiki_edit_article_content]
                        + [H - "div" + [H - "textarea.bl id=content required"]]
                        + ""]
                    + [H - "div.fh.gap"
                        + [H - "input.bl type=submit value=" - Q(lang.wiki_new_article_button)]]
                    + [H - "div id=out"]
                    + ""]
                + ""]
            + ""]
        + [H - "script"
            + compo::TIME_JS
            + "const owner="
            + owner.id()
            + ","
            + compo::fetcher_js(lang)
            + ";"
            + DirectHTML(js!(
                form.onsubmit = e => e.preventDefault() | F(fetch("/:wiki.article.new/", {
                    method: "PUT",
                    body: JSON.stringify({
                        owner,
                        title: title.value,
                        slug: slug.value,
                        content: content.value,
                    })
                }), out).then(id => {
                    onbeforeunload = 0;
                    location.href = "/_wiki_article/" + owner + "," + id;
                });
                form.oninput = _ => onbeforeunload = title.value || slug.value || content.value
                    ? _event => _event.returnValue = !_event.preventDefault()
                    : 0;
            ))]
        + ""]
    .render_page())
}
