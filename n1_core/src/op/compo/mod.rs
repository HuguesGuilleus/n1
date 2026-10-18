//! Common components for the web interface.

use n1_html::{DirectHTML, H, Html};
use n1_js::js;

use crate::op::Lang;

pub fn header(inside: impl n1_html::Html) -> impl Html {
    [H - "header.fh" + [H - "h1.bl" + inside] + [H - "a.bl.mlauto href=/_" + "Menu"] + ""]
}

/// Anchors for the header menu (at right).
/// Use with [`LOGIN_JS`] JS script.
pub fn header_menu(lang: &Lang) -> impl Html {
    [
        H + [H - "a.bl.mlauto.needNoAuth href=/_login" + lang.common_login]
            + [H - "a.bl.mlauto.mr.needAuth href=/_logout" + lang.common_logout]
            + [H - "a.bl.needAuth href=/_" + lang.common_menu],
    ]
}

pub fn footer_home(lang: &Lang) -> impl Html {
    [H - "div.mv.fh" + [H - "a.bl href=/ " + lang.common_home]]
}

/// Pass on all time element to set their datetime and text with internationalized formatting.
pub const TIME_JS: DirectHTML = DirectHTML(js!(
    document.querySelectorAll("time").forEach(_time => {
        const _date = new Date(parseInt(_time.innerText) * 1000);
        _time.datetime = _date.toJSON();
        _time.innerText = new Intl.DateTimeFormat(document.documentElement.lang, {
            dateStyle: "full",
            timeStyle: "long"
        }).format(_date);
    });
));

pub const LOGIN_JS: DirectHTML = DirectHTML(js!(
    document.body.dataset.auth=!!localStorage.getItem("auth");
    new BroadcastChannel("auth").onmessage = _ => location.reload();
));

pub const FETCHER_JS: DirectHTML = DirectHTML(js!(
    fetcher = (_promise, _out) => {
        _out.innerText = "[...]";
        _promise.then(_response => {
            if (!_response.ok ) throw _response.text();
            _out.innerText = "[OK]";
        }).catch(_err => {
            console.error(_err);
            _out.innerText = "[!!!]";
        });
    }
));
