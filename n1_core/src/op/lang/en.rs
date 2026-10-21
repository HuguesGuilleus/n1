use crate::op::Lang;

pub const LANG_EN: Lang = Lang {
    code: "en",

    common_menu: "Menu",
    common_home: "Home",
    common_save: "Save",
    common_lastedit: "Last edit: ",
    common_gotomain: "Go to main content",

    bio_edit_page: "Edit bio page of @",
    bio_edit_format_info: &[
        "Each line is analyzed separately:",
        "- as a link (starting with the URL);",
        "- as a social network link starting with 'me+https://';",
        "- as a simple paragraph;",
        "If the bio content is empty, no page are generated.",
    ],

    login_title: "Login",
    login_account: "Account or username",
    login_password: "Password",
    login_send: "Sign in",

    logout_title: "Logout",
    logout_info: "You have been logged out.",
};
