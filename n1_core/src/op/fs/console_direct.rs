use std::collections::BTreeMap;

use n1_html::{H, Html, Q, js};
use n1_tool::{DB, chuncks_len};

use crate::{
    ErrorStep, OpRequest, Result, front,
    op::{
        EntityAndObjectDTO, EntityAndObjectNullDTO, OID_ENTITY_FS, compo,
        fs::{Entry, FSState},
    },
};

pub async fn fs_direct_index(_: &impl DB, r: OpRequest<EntityAndObjectNullDTO>) -> Result<String> {
    r.token.check_access_read(r.dto.owner, OID_ENTITY_FS)?;

    let owner = r.common.get_entity(r.dto.owner).await?;
    let fs_guard = r.common.fs.get(r.dto.owner).await?;
    let fs = fs_guard.read().await;
    let path = get_full_path(&fs, owner.id(), r.dto.object)?;
    let path_str = {
        let mut s = String::new();
        for (_, name) in &path {
            s.push_str(name);
            s.push('/');
        }
        s
    };

    let children: &BTreeMap<String, u32> =
        match fs
            .entries
            .get(&r.dto.object)
            .ok_or(ErrorStep::NotFoundObject {
                owner: r.dto.owner,
                object: r.dto.object,
                type_name: "fs-entry",
            })? {
            Entry::Directory { children, .. } => children,
            Entry::File { .. } => return Err(ErrorStep::PathPartIsFile { id: r.dto.object }.into()),
        };

    let mut children = children
        .iter()
        .map(|(name, id)| -> Result<(&str, u32, &Entry)> {
            Ok((
                name.as_str(),
                *id,
                fs.entries.get(id).ok_or(ErrorStep::IncoherentFS {
                    owner: owner.id(),
                    id: *id,
                })?,
            ))
        })
        .collect::<Result<Vec<(&str, u32, &Entry)>>>()?;
    children.sort_by(|a, b| a.0.cmp(b.0));

    let lang = r.common.lang;
    let page = [H - "html lang=" - lang.code
        + [H - "head"
            + front::HEAD
            + [H - "title"
                + lang.fs_directory
                + " "
                + owner.name()
                + " "
                + (|| path.iter().map(|(_, name)| H + *name + "/"))
                + ""]
            + ""]
        + [H - "body"
            + compo::goto_main(lang)
            + [H - "header.fh.gap"
                + [H - "h1.fh.gap"
                    + [H - "div.bl" + "@" + owner.name()]
                    + || {
                        path.iter().map(|(id, name)| {
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
            + ""]
        + [H - "main id=main"
            + [H - "div.fh.gap.mv"
                + [H - "label.act.bl for=filesInput" + lang.fs_upload]
                + [
                    H - "input id=filesInput onchange=upload(event.target.files) type=file multiple hidden",
                ]
                + [H - "button.bl onclick=newDir()" + lang.fs_new_directory]
                + ""]
            + [H - "pre id=out"]
            + [H - "ul.p0"
                + [H + || {
                    children.iter().map(|&(name, id, entry)| {
                        [H - "li.fh.gap.mv data-id=" - id - " data-name=" - Q(name)
                            + [H - "a.bg href="
                                - EntityAndObjectNullDTO {
                                    owner: owner.id(),
                                    object: id,
                                }
                                + name]
                            + match entry {
                                Entry::Directory { children, .. } => {
                                    [H + "[" + children.len() + "]"]
                                }
                                Entry::File { chunks, .. } => {
                                    [H + "[" + chuncks_len(chunks) + "B]"]
                                }
                            }
                            + entry.is_file().then(|| {
                                [H - "a.bg href=/;fs.direct.get/"
                                    - EntityAndObjectDTO {
                                        owner: owner.id(),
                                        object: id,
                                    }
                                    - " download="
                                    - Q(name)
                                    + lang.common_download]
                            })
                            + [H - "button.bg onclick=rm(event)" + lang.fs_remove]
                            + ""]
                    })
                }]
                + ""]
            + ""]
        + [H - "script"
            + js! {
                const
                    owner = {extern owner.id()},
                    path = {extern path_str.as_str()},
                    {html compo::fetcher_js(lang)},
                    {html compo::NEW_ELEMENT_JS},
                    method = "PUT",
                    b = document.body,
                    rm = async event => {
                        const
                            p = event.target.parentElement,
                            d = p.dataset,
                            o = N("span", 0),
                            M = {extern lang.fs_remove} + ": " + d.name;
                        if ("popover" in HTMLElement.prototype
                            ? await new Promise(r => {
                                const p = N("div.act.bb", {popover:"auto"},
                                        {extern lang.common_remove_confirm} + ": " + d.name,
                                        N(".mt.fh.gap", 0,
                                            N("button.bl", {
                                            onclick() {
                                                p.remove();
                                            }
                                            }, {extern lang.common_cancel}),
                                            N("button.bl", {
                                                onclick() {
                                                    r(1);
                                                    p.remove();
                                                }
                                            }, {extern lang.fs_remove})
                                        ),
                                    );
                                document.body.append(p);
                                p.showPopover();
                            })
                            : confirm(M)) {
                            out.append(N("", 0, M + " => ", o));
                            F(fetch("/:fs.direct.rm", {
                                method,
                                body: JSON.stringify({
                                    owner,
                                    object: parseInt(d.id),
                                }),
                            }), o).then(_ => p.remove())
                        }
                    },
                    newDir = async _ => {
                        console.log("path", path);
                        const
                            n = prompt({extern lang.fs_new_directory}, path),
                            o = N("span", 0),
                            m = {extern lang.fs_new_directory} + ": " + n + " => ";
                        if (n && path != n) {
                            out.append(N("", 0, m, o));
                            F(fetch("/:fs.direct.mkdir", {
                                method,
                                body: JSON.stringify({
                                    owner,
                                    path: n,
                                }),
                            }), o)
                        }
                    },
                    upload = async files => {
                        const promises = [];
                        for (const f of files) {
                            const
                                o = N("span", 0, {extern lang.js_waiting}),
                                n = f.name.replace({regexp "^\\/"}, ""),
                                m = {extern lang.fs_upload} + ": " + n + " => ",
                                id = await F(fetch("/:fs.direct.mkfile", {
                                    method,
                                    body: JSON.stringify({
                                        owner,
                                        path: path + n,
                                    }),
                                }), {});
                                out.append(N("", 0, m, o));
                                promises.push(() => F(fetch("/;fs.direct.set/" + owner + "," + id, {
                                    method: "PUT",
                                    body: f,
                                }), o, 1));
                        }
                        for (const p of promises) await p();
                    },
                    upload_file = entry => {
                        if (entry.isFile) {
                            entry.file(file => upload([new File([file], entry.fullPath)]));
                        } else {
                            entry.createReader().readEntries(item => item.forEach(upload_file))
                        }
                    };
                b.ondragenter = e => e.target == b ? b.dataset.draging = 1 : 0;
                b.ondragleave = e => e.target == b ? b.dataset.draging = 0 : 0;
                b.ondragleave = e => e.target == b ? b.dataset.draging = 0 : 0;
                b.ondragover = e => e.preventDefault();
                b.ondrop = e => {
                    e.preventDefault();
                    b.dataset.draging = 0;
                    [...e.dataTransfer.items].forEach(item =>
                        item.kind == "string"
                        ? new Promise(resolve => item.getAsString(resolve))
                            .then(data => upload([new File([data], "drop_"+new Date().toJSON()+".txt")]))
                        : upload_file(item.webkitGetAsEntry())
                    );
                };
            }]
        + ""];

    Ok(page.render_page())
}

fn get_full_path(fs: &FSState, owner: u32, mut id: u32) -> Result<Vec<(u32, &str)>> {
    let mut parent = {
        let entry = fs.entries.get(&id).ok_or(ErrorStep::NotFoundObject {
            owner,
            object: id,
            type_name: "fs-entry",
        })?;
        match entry {
            Entry::Directory { parent, .. } => *parent,
            Entry::File { parent, .. } => *parent,
        }
    };

    let mut ancestors: Vec<(u32, &str)> = Vec::new();
    while id != 0 {
        let (grand_parent, children) = match fs
            .entries
            .get(&parent)
            .ok_or(ErrorStep::IncoherentFS { owner, id })?
        {
            Entry::Directory {
                parent, children, ..
            } => (parent, children),
            Entry::File { .. } => return Err(ErrorStep::IncoherentFS { owner, id }.into()),
        };

        let (name, _) = children
            .iter()
            .find(|(_, i)| **i == id)
            .ok_or(ErrorStep::IncoherentFS { owner, id })?;
        ancestors.push((id, name));

        id = parent;
        parent = *grand_parent;
    }
    ancestors.push((0u32, &""));
    ancestors.reverse();
    Ok(ancestors)
}
