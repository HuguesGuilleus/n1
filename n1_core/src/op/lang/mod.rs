//! All language translations.

mod en;
mod fr;

pub use en::LANG_EN;
pub use fr::LANG_FR;

pub struct Lang {
    /// The language code, used in HTML root lang attribute.
    pub code: &'static str,

    /// Save anchor text.
    pub common_menu: &'static str,
    /// Save button text.
    pub common_save: &'static str,
    /// Home anchor text.
    pub common_home: &'static str,
    /// Last edit label text.
    pub common_lastedit: &'static str,
    /// Anchor to the main block.
    pub common_gotomain: &'static str,

    /// Edit bio page title text.
    pub bio_edit_page: &'static str,
    /// Instruction of the bio text format.
    pub bio_edit_format_info: &'static [&'static str],

    /// Title for login page.
    pub login_title: &'static str,
    /// Account name label text.
    pub login_account: &'static str,
    /// Account password label text.
    pub login_password: &'static str,
    /// Login send button text.
    pub login_send: &'static str,

    /// Title for logout page.
    pub logout_title: &'static str,
    /// Tell the user they have been logged out.
    pub logout_info: &'static str,
}
