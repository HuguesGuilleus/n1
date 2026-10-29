use n1_html::DirectHTML;

use crate::op::Lang;

pub const LANG_FR: Lang = Lang {
    code: "fr",

    common_menu: DirectHTML("Menu"),
    common_home: DirectHTML("Accueil"),
    common_save: DirectHTML("Enregistrer"),
    common_lastedit: DirectHTML("Dernière modification\u{202F}: "),
    common_gotomain: DirectHTML("Aller au contenu"),
    common_download: DirectHTML("Télécharger"),
    common_administor: DirectHTML("Administrateur"),

    bio: DirectHTML("Bio"),
    bio_edit_page: DirectHTML("Modifier la page de bio de @"),
    bio_edit_format_info: &[
        DirectHTML("Chaque ligne est analysée séparément\u{202F}:"),
        DirectHTML("- comme un lien (commençant par l'URL)\u{00A0};"),
        DirectHTML("- comme un lien de réseau social commençant par 'me+https://'\u{00A0};"),
        DirectHTML("- comme un simple paragraphe\u{00A0};"),
        DirectHTML("Si le contenu de la bio est vide, aucune page n'est générée."),
    ],

    fs: DirectHTML("Fichiers"),
    fs_remove: DirectHTML("Supprimer"),
    fs_upload: DirectHTML("Téléverser"),

    login_title: DirectHTML("Connexion"),
    login_account: DirectHTML("Compte ou pseudo"),
    login_password: DirectHTML("Mot de passe"),
    login_send: "Se connecter",

    logout_title: DirectHTML("Déconnexion"),
    logout_info: DirectHTML("Vous avez été déconnecté."),

    wiki: DirectHTML("Wiki"),
};
