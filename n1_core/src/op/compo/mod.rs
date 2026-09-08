use n1_html::{DirectHTML, H};

pub fn header(inside: impl n1_html::Html) -> impl n1_html::Html {
    [H - "header.fh" + [H - "h1.bl" + inside] + [H - "a.bl.mlauto href=/_" + "Menu"] + ""]
}

pub const TIME_JS: DirectHTML = DirectHTML(
    r#"document.querySelectorAll("time").forEach(t=>t.innerText=new Intl.DateTimeFormat(document.documentElement.lang,{dateStyle:"full",timeStyle:"long"}).format(new Date(parseInt(t.innerText)*1000)));"#,
);

pub const LOGIN_JS: DirectHTML =
    DirectHTML(r#"(localStorage.getItem("isauth")?logout:login).remove();"#);
