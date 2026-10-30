use n1_html::DirectHTML;

use crate::op::Lang;

pub const LANG_EN: Lang = Lang {
    code: "en",

    auth_admin_global: DirectHTML("Global administration"),

    common_menu: DirectHTML("Menu"),
    common_home: DirectHTML("Home"),
    common_save: DirectHTML("Save"),
    common_lastedit: DirectHTML("Last edit: "),
    common_gotomain: DirectHTML("Go to main content"),
    common_download: DirectHTML("Download"),
    common_administor: DirectHTML("Administrator"),

    bio: DirectHTML("Bio"),
    bio_edit_page: DirectHTML("Edit bio page of @"),
    bio_edit_format_info: &[
        DirectHTML("Each line is analyzed separately:"),
        DirectHTML("- as a link (starting with the URL);"),
        DirectHTML("- as a social network link starting with 'me+https://';"),
        DirectHTML("- as a simple paragraph;"),
        DirectHTML("If the bio content is empty, no page are generated."),
    ],

    fs: DirectHTML("Files"),
    fs_remove: DirectHTML("Remove"),
    fs_upload: DirectHTML("Upload"),

    home: DirectHTML("Home"),
    home_title: DirectHTML("Modification de l'accueil"),
    home_edite_title: DirectHTML("Title:"),
    home_edite_description: DirectHTML("Description:"),
    home_page_content: DirectHTML("Home page content, use blank line to separt paragraph:"),

    login_title: DirectHTML("Login"),
    login_account: DirectHTML("Account or username"),
    login_password: DirectHTML("Password"),
    login_send: "Sign in",

    logout_title: DirectHTML("Logout"),
    logout_info: DirectHTML("You have been logged out."),

    wiki: DirectHTML("Wiki"),
};
