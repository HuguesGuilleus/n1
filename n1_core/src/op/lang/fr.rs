use crate::op::Lang;

pub const LANG_FR: Lang = Lang {
    code: "fr",

    common_menu: n1_html::DirectHTML("Menu"),
    common_home: n1_html::DirectHTML("Accueil"),
    common_save: n1_html::DirectHTML("Enregistrer"),
    common_lastedit: n1_html::DirectHTML("Dernière modification\u{202F}: "),
    common_gotomain: n1_html::DirectHTML("Aller au contenu"),

    bio_edit_page: n1_html::DirectHTML("Modifier la page de bio de @"),
    bio_edit_format_info: &[
        n1_html::DirectHTML("Chaque ligne est analysée séparément\u{202F}:"),
        n1_html::DirectHTML("- comme un lien (commençant par l'URL)\u{00A0};"),
        n1_html::DirectHTML(
            "- comme un lien de réseau social commençant par 'me+https://'\u{00A0};",
        ),
        n1_html::DirectHTML("- comme un simple paragraphe\u{00A0};"),
        n1_html::DirectHTML("Si le contenu de la bio est vide, aucune page n'est générée."),
    ],

    login_title: n1_html::DirectHTML("Connexion"),
    login_account: n1_html::DirectHTML("Compte ou pseudo"),
    login_password: n1_html::DirectHTML("Mot de passe"),
    login_send: "Se connecter",

    logout_title: n1_html::DirectHTML("Déconnexion"),
    logout_info: n1_html::DirectHTML("Vous avez été déconnecté."),
};
