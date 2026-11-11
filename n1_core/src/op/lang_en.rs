use n1_html::DirectHTML;

use super::Lang;

pub const LANG_EN: Lang = Lang {
    code: "en",

    auth_admin_global: DirectHTML("Global administration"),

    common_administor: DirectHTML("Administrator"),
    common_download: DirectHTML("Download"),
    common_edit: DirectHTML("Edit"),
    common_gotomain: DirectHTML("Go to main content"),
    common_home: DirectHTML("Home"),
    common_lastedit: DirectHTML("Last edit: "),
    common_menu: DirectHTML("Menu"),
    common_remove: DirectHTML("Remove"),
    common_remove_confirm: DirectHTML("Are you sure you want to remove?"),
    common_save: DirectHTML("Save"),

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
    fs_directory: DirectHTML("Directory"),
    fs_new_directory: DirectHTML("Create a new directory"),
    fs_remove: DirectHTML("Remove"),
    fs_upload: DirectHTML("Upload"),

    home: DirectHTML("Home"),
    home_title: DirectHTML("Modification de l'accueil"),
    home_edit_title: DirectHTML("Title:"),
    home_edit_description: DirectHTML("Description:"),
    home_page_content: DirectHTML("Home page content, use blank line to separt paragraph:"),

    js_ok: DirectHTML("OK"),
    js_error: DirectHTML("Error: "),
    js_waiting: DirectHTML("Waiting..."),

    login_title: DirectHTML("Login"),
    login_username: DirectHTML("Account or username"),
    login_password: DirectHTML("Password"),
    login_send: "Sign in",

    logout_title: DirectHTML("Logout"),
    logout_info: DirectHTML("You have been logged out."),

    wiki: DirectHTML("Wiki"),
    wiki_new_article: DirectHTML("New article"),
    wiki_edit_article: DirectHTML("Edit article"),
    wiki_new_article_button: DirectHTML("Add a new article"),
    wiki_edit_article_title: DirectHTML("Title:"),
    wiki_edit_article_slug: DirectHTML("Slug (used in URL):"),
    wiki_edit_article_content: DirectHTML("Content:"),
    wiki_public_anchor: DirectHTML("Public article"),
    wiki_public_zone: DirectHTML("Public zone"),
};
