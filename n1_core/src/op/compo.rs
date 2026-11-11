//! Common components for the web interface.

use n1_html::{DirectHTML, H, Html, Q};
use n1_js::js;

use super::Lang;

/// Create link to `#main` for accessibility.
pub fn goto_main(lang: &Lang) -> impl Html {
    [H - "a.bl href=#main" + lang.common_gotomain]
}

/// Anchors for the header menu (at right).
/// Use with [`LOGIN_JS`] JS script.
pub fn header_menu(lang: &Lang) -> impl Html {
    [
        H + [H - "a.bl.mlauto.needNoAuth href=/_auth_login" + lang.login_title]
            + [H - "a.bl.mlauto.needAuth href=/_auth_logout" + lang.logout_title]
            + [H - "a.bl.needAuth href=/_" + lang.common_menu],
    ]
}

/// Anchors for the header menu (at right).
/// Assume that user is logged.
/// No need additional script.
pub fn header_menu_logged(lang: &Lang) -> impl Html {
    [
        H + [H - "a.bl.mlauto href=/_auth_logout" + lang.logout_title]
            + [H - "a.bl.needAuth href=/_" + lang.common_menu],
    ]
}

/// Add link to home page for the public page footer.
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

/// Add to js to manage user authentication state.
pub const LOGIN_JS: DirectHTML = DirectHTML(js!(
    document.body.dataset.auth=this.auth=!!localStorage.getItem("auth");
    new BroadcastChannel("auth").onmessage = _ => location.reload();
));

/// Add JS to manage login/logout event.
pub const LOGIN_JS_CONNECTED: DirectHTML = DirectHTML(js!(
    new BroadcastChannel("auth").onmessage = _ => location.reload();
));

/// JS script to fetch api and indicate when request end and his sttaus.
pub const FETCHER_JS: DirectHTML = DirectHTML(js!(
    fetcher = (_promise, _out) => (
        _out.innerText = "[...]",
        _promise.then(async _response => {
            if (!_response.ok ) throw await _response.text();
            _out.innerText = "[OK]";
        }).catch(_err => {
            _out.innerText = "[!!!] " + _err;
            throw _err;
        }),
    )
));

/// Define `F` a JS function to handle request promise.
/// Define it within a constant declaration.
pub fn fetcher_js(lang: &'static Lang) -> impl Html {
    [H + "F=(p,o,"
        + [H + "W=" + Q(lang.js_waiting) + ","]
        + [H + "O=" + Q(lang.js_ok) + ","]
        + [H + "E=" + Q(lang.js_error)]
        + DirectHTML(")=>")
        + DirectHTML(js!({
            o.innerText = W;
            return p.then(async r => {
                if (r.ok) return o.innerText = O, r.json();
                else throw await r.text()
            }).catch(e=>{o.innerText = E + e; throw e;});
        }))]
}
