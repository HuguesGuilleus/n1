use n1_html::{DirectHTML, H, Html, Q};
use n1_js::js;
use n1_tool::{DB, chuncks_len};

use crate::{
    OpRequest, Result, errs, front,
    op::{EntityAndObjectDTO, EntityAndObjectNullDTO, OID_ENTITY_FS, compo, fs::Entry},
};

pub async fn fs_direct_index(_: &impl DB, r: OpRequest<EntityAndObjectNullDTO>) -> Result<String> {
    r.token.check_access_read(r.dto.owner, OID_ENTITY_FS)?;

    let fs_guard = r.common.fs.get(r.dto.owner).await?;
    let fs = fs_guard.read().await;

    let (parent, dir) = fs
        .dirs
        .get(&r.dto.object)
        .ok_or_else(|| errs::NOT_FOUND.push(format!("do not find directory {}", r.dto.object)))?;
    let mut dir: Vec<&Entry> = dir.iter().collect();
    dir.sort_by_key(|entry| &entry.name);

    let mut ancestors: Vec<(u32, &str)> = vec![];
    let mut p = r.dto.object;
    let mut parent = *parent;
    while p != 0 {
        let (grandparent, entries) = fs
            .dirs
            .get(&parent)
            .ok_or_else(|| errs::DATA_INCOHERENT.push(format!("cannot find parent of {}", p)))?;
        let entry = entries
            .iter()
            .filter(|entry| entry.id == p)
            .next()
            .ok_or_else(|| {
                errs::DATA_INCOHERENT.push(format!("In {} cannot find directory {}", parent, p))
            })?;
        ancestors.push((p, &entry.name));
        p = parent;
        parent = *grandparent;
    }
    ancestors.push((0u32, &""));
    ancestors.reverse();

    let lang = r.common.lang;
    let page = [H - "html lang=" - lang.code
        + [H - "head" + front::HEAD + [H - "title" + "!Répertoire " + ""]]
        + [H - "body"
            + compo::goto_main(lang)
            + [H - "header.fh"
                + [H - "h1.fh"
                    + || {
                        ancestors.iter().map(|(id, name)| {
                            [H - "a.bl href="
                                - EntityAndObjectNullDTO {
                                    owner: r.dto.owner,
                                    object: *id,
                                }
                                + *name
                                + "/"]
                        })
                    }]
                + compo::header_menu_logged(lang)
                + ""]
            + [H - "main id=main"
                + "$$upload + drop"
                + [H - "div id=out"]
                + [H - "ul"
                    + [H + || {
                        dir.iter().map(|entry| {
                            [H - "li.fh.gap data-id=" - entry.id
                                + [H - "a.bg href="
                                    - entry.is_dir.then_some("/_fs_index/").unwrap_or("/fs_file/")
                                    - EntityAndObjectNullDTO {
                                        owner: r.dto.owner,
                                        object: entry.id,
                                    }
                                    + &entry.name
                                    + entry.is_dir.then_some("/")
                                    + (!entry.is_dir).then(|| {
                                        fs.files.get(&entry.id).map(|(_, chunks)| {
                                            [H + " (" + chuncks_len(chunks) + "B)"]
                                        })
                                    })
                                    + ""]
                                + (!entry.is_dir).then(|| {
                                    H - "a.bg href=/;fs.direct.get/"
                                        - EntityAndObjectDTO {
                                            owner: r.dto.owner,
                                            object: entry.id,
                                        }
                                        - " title="
                                        - Q(lang.common_download)
                                        + "dl"
                                })
                                + [H - "button.bg onclick=rm(event) title=" - Q(lang.fs_remove)
                                    + "rm"]
                                + ""]
                        })
                    }]]
                + ""]
            + ""]
        + [H - "script"
            + "const owner="
            + r.dto.owner
            + ","
            + compo::FETCHER_JS
            + ","
            + DirectHTML(js!(
                rm = _event => {
                    fetcher(fetch("/:fs.direct.rm", {
                        method: "PUT",
                        body: JSON.stringify({
                            owner,
                            object: parseInt(_event.target.parentElement.dataset.id),
                        }),
                    }).then(_event.target.parentElement.remove()), out)
                };
            ))
            + ""]]
    .render_page();

    Ok(page)
}
