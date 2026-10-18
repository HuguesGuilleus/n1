use crate::op::Lang;

pub const LANG_EN: Lang = Lang {
    code: "en",

    common_menu: "Menu",
    common_login: "Login",
    common_logout: "Logout",
    common_home: "Home",
    common_save: "Save",
    common_lastedit: "Last edit: ",

    bio_edit_page: "Edit bio page of @",
    bio_edit_format_info: &[
        "Each line is analyzed separately:",
        "- as a link (starting with the URL);",
        "- as a social network link starting with 'me+https://';",
        "- as a simple paragraph;",
        "If the bio content is empty, no page are generated.",
    ],
};
