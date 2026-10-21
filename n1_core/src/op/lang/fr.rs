use crate::op::Lang;

pub const LANG_FR: Lang = Lang {
    code: "fr",

    common_menu: "Menu",
    common_home: "Accueil",
    common_save: "Enregistrer",
    common_lastedit: "Dernière modification\u{202F}: ",
    common_gotomain: "Aller au contenu",

    bio_edit_page: "Modifier la page de bio de @",
    bio_edit_format_info: &[
        "Chaque ligne est analysée séparément\u{202F}:",
        "- comme un lien (commençant par l'URL)\u{00A0};",
        "- comme un lien de réseau social commençant par 'me+https://'\u{00A0};",
        "- comme un simple paragraphe\u{00A0};",
        "Si le contenu de la bio est vide, aucune page n'est générée.",
    ],

    login_title: "Connexion",
    login_account: "Compte ou pseudo",
    login_password: "Mot de passe",
    login_send: "Se connecter",

    logout_title: "Déconnexion",
    logout_info: "Vous avez été déconnecté.",
};
