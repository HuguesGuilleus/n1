//! All language translations.

mod en;
mod fr;

pub use en::LANG_EN;
pub use fr::LANG_FR;
use n1_html::DirectHTML;

pub struct Lang {
    /// The language code, used in HTML root lang attribute.
    pub code: &'static str,

    /// Save anchor text.
    pub common_menu: DirectHTML<'static>,
    /// Save button text.
    pub common_save: DirectHTML<'static>,
    /// Home anchor text.
    pub common_home: DirectHTML<'static>,
    /// Last edit label text.
    pub common_lastedit: DirectHTML<'static>,
    /// Anchor to the main block.
    pub common_gotomain: DirectHTML<'static>,
    /// Help text of download button.
    pub common_download: DirectHTML<'static>,

    /// Edit bio page title text.
    pub bio_edit_page: DirectHTML<'static>,
    /// Instruction of the bio text format.
    pub bio_edit_format_info: &'static [DirectHTML<'static>],

    /// Help text to remove a file or a directory.
    pub fs_remove: DirectHTML<'static>,

    /// Title for login page.
    pub login_title: DirectHTML<'static>,
    /// Account name label text.
    pub login_account: DirectHTML<'static>,
    /// Account password label text.
    pub login_password: DirectHTML<'static>,
    /// Login send button text.
    pub login_send: &'static str,

    /// Title for logout page.
    pub logout_title: DirectHTML<'static>,
    /// Tell the user they have been logged out.
    pub logout_info: DirectHTML<'static>,
}
