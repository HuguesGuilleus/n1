use crate::{op::user::Entity, *};
use n1_html::{H, Html};
use n1_tool::Config;

pub async fn page(server: &OpServer<impl Config>, r: OpRequest<()>) -> Result<String> {
    r.token.is_auth()?;
    let entities = server.entities.read()?;
    let entity = entities.get(&r.token.uid).ok_or(errs::NOT_FOUND_USER)?;
    let user = if let Entity::User(user) = entity {
        user
    } else {
        return Err(errs::EXPECT_USER);
    };

    Ok([H - "html lang=fr"
        + [H - "head" + front::HEAD + [H - "title" + "Menu"]]
        + [H - "body"
            + op::compo::header(
                true,
                H - "" + "Menu @" + &user.name + " [" + user.global.as_str() + "]",
            )
            + [H - "main.w"
                + [H - "h1" + "Utilisateur"]
                + [H - "div.fh.gap"
                    + [H - "a.bl href=/_bio/" - user.uid + "bio"]
                    + [H - "a.bl href=/_dropbox/" - user.uid + "fichiers"]
                    + [H - "a.bl href=/_dropbox/" - user.uid + "dépôt"]
                    + [H - "a.bl href=/_mail/" - user.uid + "mail"]
                    + ""]
                + (|| {
                    user.groups().map(|(gid, _)| {
                        [H + [H - "h1"
                            + "Groupe: "
                            + entities
                                .get(&gid)
                                .map(|entity| entity.name())
                                .unwrap_or_default()]
                            + [H - "div.fh.gap"
                                + [H - "a.bl href=/_bio/" - gid + "bio"]
                                + [H - "a.bl href=/_dropbox/" - gid + "fichiers"]
                                + [H - "a.bl href=/_dropbox/" - gid + "dépôt"]
                                + [H - "a.bl href=/_mail/" - gid + "mail"]
                                + ""]
                            + ""]
                    })
                })
                + [H - "h1" + "Administration"]
                + [H - "div.fh.gap.mv"
                    + [H - "a.bl href=/_users" + "Utilisateurs"]
                    + [H - "a.bl href=/_groups" + "Groupes"]
                    + ""]
                + [H - "div.fh.gap"
                    + [H - "button.bl" + "Nouveau mot de passe"]
                    + [H - "button.bl" + "Supprimer le compte"]
                    + ""]
                + ""]
            + ""]
        + ""]
    .render_page())
}
