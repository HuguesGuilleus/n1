use std::{str::FromStr, sync::Arc};

use n1_tool::DB;

use crate::{
    op::{
        bio,
        fs::{fs_direct_mkdir, fs_direct_mkfile, fs_direct_set, MakeDTO},
        wiki, Entity, EntityAndObjectDTO, Group, Name, Token, TokenItem, User, OID_ENTITY_BIO,
        OID_ENTITY_DROPBOX, OID_ENTITY_FS, OID_ENTITY_WIKI, OID_GLOBAL_ENTITY, OID_GLOBAL_HOME,
    },
    Common, OpRequest, Result,
};

/// Add data for development purposes.
pub async fn init_dev(db: &impl DB, common: Arc<Common>) -> Result<()> {
    // Set entities
    {
        let mut entities_map = common.entities.write().await;
        entities_map.insert(
            1,
            Entity::Organization(crate::op::Organization {
                id: 1,
                name: "syndicate".to_string(),
                entities: std::collections::BTreeSet::new(),
                default_access: vec![],
            }),
        );

        entities_map.insert(
            101,
            Entity::User(User {
                uid: 101,
                name: "eve".to_string(),
                password: "56".to_string(),
                is_admin: true,
                groups: vec![],
                organization: 1,
            }),
        );

        entities_map.insert(
            201,
            Entity::Group(Group {
                gid: 201,
                name: "world".to_string(),
                users: vec![
                    TokenItem {
                        id: 102,
                        app: OID_ENTITY_FS,
                        can_write: false,
                    },
                    TokenItem {
                        id: 102,
                        app: OID_ENTITY_WIKI,
                        can_write: true,
                    },
                ],
                organization: 1,
            }),
        );
    }

    init_sarah_data(db, common.clone()).await;

    Ok(())
}

async fn init_sarah_data(db: &impl DB, common: Arc<Common>) {
    let uid = 103;
    common.entities.write().await.insert(
        uid,
        Entity::User(User {
            uid,
            name: "sarah".to_string(),
            password: "21".to_string(),
            is_admin: false,
            groups: vec![
                TokenItem {
                    id: uid,
                    app: OID_ENTITY_FS,
                    can_write: true,
                },
                TokenItem {
                    id: uid,
                    app: OID_ENTITY_BIO,
                    can_write: true,
                },
                TokenItem {
                    id: uid,
                    app: OID_ENTITY_WIKI,
                    can_write: true,
                },
                TokenItem {
                    id: 201,
                    app: OID_ENTITY_FS,
                    can_write: false,
                },
            ],
            organization: 1,
        }),
    );

    let token = Token {
        uid,
        is_admin: false,
        access: {
            let mut access = [TokenItem::default(); Token::ACCESS_LEN];
            access[0] = TokenItem::new_write(uid, OID_GLOBAL_ENTITY);
            access[1] = TokenItem::new_write(uid, OID_GLOBAL_HOME);
            access[2] = TokenItem::new_write(uid, OID_ENTITY_FS);
            access[3] = TokenItem::new_write(uid, OID_ENTITY_BIO);
            access[4] = TokenItem::new_write(uid, OID_ENTITY_DROPBOX);
            access[5] = TokenItem::new_write(uid, OID_ENTITY_WIKI);
            access
        },
    };

    // Bio
    bio::set(
        db,
        OpRequest {
            common: common.clone(),
            token: token.clone(),
            dto: bio::BioSetDTO {
                owner: uid,
                content: concat!(
                    "me+https://github.com/octocat Octocat\n",
                    "me+https://octocat.github.io Website\n",
                    "https://github.com/octocat/Hello-World My hello world example\n",
                    "https://github.blog\n",
                    "Hello World ...",
                )
                .to_string(),
            },
        },
    )
    .await
    .unwrap();

    // File system
    fs_direct_mkdir(
        db,
        OpRequest {
            common: common.clone(),
            token: token.clone(),
            dto: MakeDTO {
                owner: uid,
                path: "dir/sub/".to_string(),
            },
        },
    )
    .await
    .unwrap();

    let hello_id = fs_direct_mkfile(
        db,
        OpRequest {
            common: common.clone(),
            token: token.clone(),
            dto: MakeDTO {
                owner: uid,
                path: "dir/sub/hello.txt".to_string(),
            },
        },
    )
    .await
    .unwrap();

    fs_direct_set(
        db,
        OpRequest {
            common: common.clone(),
            token: token.clone(),
            dto: EntityAndObjectDTO {
                owner: uid,
                object: hello_id,
            },
        },
        &b"Hello World\r\n"[..],
    )
    .await
    .unwrap();

    // Wiki
    wiki::article_new(
        db,
        OpRequest {
            common: common.clone(),
            token: token.clone(),
            dto: wiki::NewArticleDTO {
                owner: uid,
                title: "Hello Wiki".to_string(),
                slug: Name::from_str("hello").unwrap(),
                content: "Hello Wiki\r\n".to_string(),
            },
        },
    )
    .await
    .unwrap();
}
