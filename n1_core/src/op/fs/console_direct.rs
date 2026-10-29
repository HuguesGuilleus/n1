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
                + [H - "div.fh.gap.mv"
                    + [H -"label.act.bl for=filesInput" + lang.fs_upload  ]
                    + [H -"input id=filesInput onchange=upload(event.target.files) type=file multiple hidden"  ]
                + ""]
                + [H - "pre id=out"]
                + [H - "ul.p0"
                    + [H + || {
                        dir.iter().map(|entry| {
                            [H - "li.fh.gap data-id=" - entry.id - " data-name=" - Q(&entry.name)
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
            + "path="
            + Q([H + || ancestors.iter().flat_map(|(_, name)| [*name,"/"])])
            + ","
            + DirectHTML(js!(
                _newElement = (type, _obj, ..._children)=> {
                    const [_tagName, ..._classNames] = type.split(r#"regexp/\.(\w+)/"#),
                        _element = document.createElement(_tagName || "div");
                    _element.classList.add(..._classNames.filter(x => x));
                    _element.append(..._children);
                    Object.assign(_element, _obj || {});
                    return _element;
                },
                upload = async _files => {
                    for (const _file of _files) {
                        const _out = _newElement("", 0, "+ ", _file.name, " ... ");
                        out.append(_out);
                        const _response1 = await fetch("/:fs.direct.mkfile", {
                            method: "PUT",
                            body: JSON.stringify({
                                owner,
                                path: path + _file.name.replace(r#"regexp/^\//"#, ""),
                            }),
                        });
                        if (!_response1.ok) return _out.innerText += "!!! " + await _response1.text();
                        const _id = await _response1.json();
                        _out.innerText += "#" + _id + " ... ";
                        const _response2 = await fetch("/;fs.direct.set/" + owner + "," + _id, {
                            method: "PUT",
                            body: _file,
                        });
                        _out.innerText += _response2.ok ? "OK" : "!!! " + await _response2.text();
                    }
                },
                rm = _event => {
                    const
                        _data = _event.target.parentElement.dataset,
                        _out = _newElement("", 0, "- ", _data.name, " ... ");
                    out.append(_out);
                    fetch("/:fs.direct.rm", {
                        method: "PUT",
                        body: JSON.stringify({
                            owner,
                            object: parseInt(_data.id),
                        }),
                    }).then(r => r.ok ? "OK" : r.text())
                    .catch(_err => "!!! " + _err)
                    .then(r => _out.textContent += r);
                },
                _body = document.body;

                _body.ondragover = _event => _event.preventDefault();
                _body.ondrop = _event => {
                    _event.preventDefault();
                    const _upload_file = async _entry => {
                        if (_entry.isFile) {
                            upload([
                                new File([
                                    await new Promise(_resolve => _entry.file(_resolve))
                                ], _entry.fullPath)
                            ]);
                        } else {
                            (await new Promise(
                                _resolve => _entry.webkitGetAsEntry().createReader().readEntries(_resolve)
                            )).forEach(_upload_file);
                        }
                    };
                    [..._event.dataTransfer.items].forEach(_item => _item.kind == "string"
                        ? new Promise(_resolve => _item.getAsString(_resolve))
                            .then(_data => upload([new File([_data], "drop_"+new Date().toJSON()+".txt")]))
                        : _upload_file(_item)
                    );
                };
            ))
            + ""]]
    .render_page();

    Ok(page)
}
