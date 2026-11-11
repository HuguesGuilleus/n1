use n1_html::DirectHTML;

use super::Lang;

pub const LANG_FR: Lang = Lang {
    code: "fr",

    auth_admin_global: DirectHTML("Administration globale"),

    common_administor: DirectHTML("Administrateur"),
    common_download: DirectHTML("Télécharger"),
    common_edit: DirectHTML("Modifier"),
    common_gotomain: DirectHTML("Aller au contenu"),
    common_home: DirectHTML("Accueil"),
    common_lastedit: DirectHTML("Dernière modification\u{202F}: "),
    common_menu: DirectHTML("Menu"),
    common_remove: DirectHTML("Supprimer"),
    common_remove_confirm: DirectHTML("Êtes-vous sûr de vouloir supprimer\u{00A0}?"),
    common_save: DirectHTML("Enregistrer"),

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
    fs_directory: DirectHTML("Répertoire"),
    fs_new_directory: DirectHTML("Créer un nouveau répertoire"),
    fs_remove: DirectHTML("Supprimer"),
    fs_upload: DirectHTML("Téléverser"),

    home: DirectHTML("Accueil"),
    home_title: DirectHTML("Home editor"),
    home_edit_title: DirectHTML("Titre\u{00A0}:"),
    home_edit_description: DirectHTML("Description\u{00A0}:"),
    home_page_content: DirectHTML(
        "Contenu de la page d'accueil, utilisez des lignes vides pour séparer les paragraphes\u{202F}:",
    ),

    js_ok: DirectHTML("OK"),
    js_error: DirectHTML("Erreur\u{00A0}: "),
    js_waiting: DirectHTML("En attente..."),

    login_title: DirectHTML("Connexion"),
    login_username: DirectHTML("Compte ou pseudo"),
    login_password: DirectHTML("Mot de passe"),
    login_send: "Se connecter",

    logout_title: DirectHTML("Déconnexion"),
    logout_info: DirectHTML("Vous avez été déconnecté."),

    wiki: DirectHTML("Wiki"),
    wiki_edit_article: DirectHTML("Modification d'un article"),
    wiki_new_article_button: DirectHTML("Ajouter un nouvel article"),
    wiki_edit_article_title: DirectHTML("Titre\u{00A0}:"),
    wiki_edit_article_slug: DirectHTML("Identifiant de l'URL\u{00A0}:"),
    wiki_edit_article_content: DirectHTML("Contenu\u{00A0}:"),
    wiki_new_article: DirectHTML("Nouvel article"),
    wiki_public_anchor: DirectHTML("Article public"),
    wiki_public_zone: DirectHTML("Partie publique"),
};
