use std::ops::Not;

use n1_html::{DirectHTML, H};

pub fn header(auth: bool, inside: impl n1_html::Html) -> impl n1_html::Html {
    [H - "header.fh"
        + [H - "a.bl href=/_" + "///"]
        + [H - "div.bl" + inside]
        + auth.then_some(H - "a.bl.mlauto id=logout href=/_logout" + "Déconnexion")
        + auth.not().then_some(
            H + [H - "a.bl.mlauto id=login href=/_login" + "Connexion"]
                + [H - "a.bl.mlauto id=logout href=/_logout" + "Déconnexion"]
                + [H - "script"
                    + DirectHTML(r#"(localStorage.getItem("isauth")?login:logout).hidden=!0"#)],
        )
        + ""]
}
