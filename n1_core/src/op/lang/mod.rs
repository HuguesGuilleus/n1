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
    /// Login anchor text.
    pub common_login: &'static str,
    /// Logout anchor text.
    pub common_logout: &'static str,
    /// Save button text.
    pub common_save: &'static str,
    /// Home anchor text.
    pub common_home: &'static str,
    /// Last edit label text.
    pub common_lastedit: &'static str,

    /// Edit bio page title text.
    pub bio_edit_page: &'static str,
    /// Instruction of the bio text format.
    pub bio_edit_format_info: &'static [&'static str],
}
