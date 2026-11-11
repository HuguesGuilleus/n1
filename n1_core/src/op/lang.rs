use n1_html::DirectHTML;

/// All language translations.
pub struct Lang {
    /// The language code, used in HTML root lang attribute.
    pub code: &'static str,

    pub auth_admin_global: DirectHTML<'static>,

    /// Slug of administrator.
    pub common_administor: DirectHTML<'static>,
    /// Help text of download button.
    pub common_download: DirectHTML<'static>,
    /// Edit the page button text.
    pub common_edit: DirectHTML<'static>,
    /// Anchor to the main block.
    pub common_gotomain: DirectHTML<'static>,
    /// Home anchor text.
    pub common_home: DirectHTML<'static>,
    /// Last edit label text.
    pub common_lastedit: DirectHTML<'static>,
    /// Save anchor text.
    pub common_menu: DirectHTML<'static>,
    /// Remove button text.
    pub common_remove: DirectHTML<'static>,
    /// Confirmation message before removing an item.
    pub common_remove_confirm: DirectHTML<'static>,
    /// Save button text.
    pub common_save: DirectHTML<'static>,

    /// Bio console name.
    pub bio: DirectHTML<'static>,
    /// Edit bio page title text.
    pub bio_edit_page: DirectHTML<'static>,
    /// Instruction of the bio text format.
    pub bio_edit_format_info: &'static [DirectHTML<'static>],

    /// File system name.
    pub fs: DirectHTML<'static>,
    /// Title for directory.
    pub fs_directory: DirectHTML<'static>,
    /// Create a new directory.
    pub fs_new_directory: DirectHTML<'static>,
    /// Help text to remove a file or a directory.
    pub fs_remove: DirectHTML<'static>,
    /// Label text to remove a file or a directory.
    pub fs_upload: DirectHTML<'static>,

    /// Home anchor.
    pub home: DirectHTML<'static>,
    /// Home console title.
    pub home_title: DirectHTML<'static>,
    /// Label edit the home title.
    pub home_edit_title: DirectHTML<'static>,
    /// Label edit the home dscription.
    pub home_edit_description: DirectHTML<'static>,
    /// Label edit the page content.
    pub home_page_content: DirectHTML<'static>,

    /// Message to display when response successful.
    pub js_ok: DirectHTML<'static>,
    /// Message to display when an error occurs.
    pub js_error: DirectHTML<'static>,
    /// Message to display while waiting for a response.
    pub js_waiting: DirectHTML<'static>,

    /// Title for login page.
    pub login_title: DirectHTML<'static>,
    /// Account name label text.
    pub login_username: DirectHTML<'static>,
    /// Account password label text.
    pub login_password: DirectHTML<'static>,
    /// Login send button text.
    pub login_send: &'static str,

    /// Title for logout page.
    pub logout_title: DirectHTML<'static>,
    /// Tell the user they have been logged out.
    pub logout_info: DirectHTML<'static>,

    /// Title for wiki.
    pub wiki: DirectHTML<'static>,
    /// Add a new page title text.
    pub wiki_new_article: DirectHTML<'static>,
    /// Edit the page title text.
    pub wiki_edit_article: DirectHTML<'static>,
    /// Add a new page button text.
    pub wiki_new_article_button: DirectHTML<'static>,
    /// Title for the new article page input field.
    pub wiki_edit_article_title: DirectHTML<'static>,
    /// Slug for the new article page input field.
    pub wiki_edit_article_slug: DirectHTML<'static>,
    /// Content for the new article page input field.
    pub wiki_edit_article_content: DirectHTML<'static>,
    /// Public anchor to the published article.
    pub wiki_public_anchor: DirectHTML<'static>,
    /// Public zone wiki label text.
    pub wiki_public_zone: DirectHTML<'static>,
}
