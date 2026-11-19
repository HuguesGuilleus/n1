use n1_html::{Html, H};
use n1_tool::proto_http::StatusHTTP;

use crate::{front, Error};

pub fn print_error_html(err: Error, status: StatusHTTP) -> String {
    [H - "html lang=en"
        + [H - "head" + front::HEAD]
        + [H - "body.msg"
            + [H - "main.w.fv.gap"
                + [H - "h1.bl" + status.as_str()]
                + [|| err.0.iter().map(|s| [H - "div" + "--- " + s.to_string()])]
                + [H - "div.fh.gap"
                    + [H - "a.bl href=/_ " + "Menu"]
                    + [H - "a.bl href=/_auth_login" + "Login"]
                    + [H - "a.bl href=/_auth_logout" + "Logout"]
                    + ""]
                + ""]
            + ""]
        + ""]
    .render_page()
}
