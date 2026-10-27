use std::{collections::BTreeMap, sync::Arc};

use bytes::Bytes;
use n1_tool::{Chunk, DB, DBMemory, Result};

use crate::{
    Common, OpRequest,
    op::{
        self, EntityAndObjectDTO, OID_ENTITY_BIO, OID_ENTITY_DROPBOX, OID_ENTITY_FS,
        OID_ENTITY_WIKI, OID_GLOBAL_ENTITY, OID_GLOBAL_HOME, Token, TokenItem,
        dropbox::DropFile,
        fs::{MakeDTO, fs_direct_mkdir, fs_direct_mkfile, fs_direct_set},
        user::{Entity, Group, User},
    },
};

const NOW: u64 = 1785597192;

pub async fn init_dev() -> Result<(impl DB, Arc<Common>)> {
    let db = DBMemory::new();
    let mut common = Common::new();

    common.lang = &crate::op::LANG_FR;

    // Set "eve" user
    let entities_map = common.entities.get_mut();
    entities_map.insert(
        101,
        Entity::User(User {
            uid: 101,
            name: "eve".to_string(),
            password: "56".to_string(),
            is_admin: true,
            access: vec![],
        }),
    );
    db.obj_set(
        101,
        OID_ENTITY_BIO as u32,
        op::bio::BioState {
            last_edit: NOW,
            content: concat!(
                "me+https://github.com/octocat Octocat\n",
                "me+https://octocat.github.io Website\n",
                "https://github.com/octocat/Hello-World My hello world example\n",
                "https://github.blog\n",
                "Hello World ...",
            )
            .to_string(),
        },
    )
    .await?;

    // Set bob user
    entities_map.insert(
        102,
        Entity::User(User {
            uid: 102,
            name: "bob".to_string(),
            password: "42".to_string(),
            is_admin: false,
            access: vec![
                TokenItem {
                    id: 201,
                    app: OID_ENTITY_FS,
                    can_write: false,
                },
                TokenItem {
                    id: 201,
                    app: OID_ENTITY_WIKI,
                    can_write: true,
                },
            ],
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
        }),
    );

    // Set dropbox
    db.obj_set(101 , OID_ENTITY_DROPBOX as u32 , op::dropbox::State{
        shadow: 301,
        texts_inc: 3,
        texts: vec![
            (0, "Text 1".to_string()),
            (2, "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed non risus. Suspendisse lectus tortor, dignissim sit amet, adipiscing nec, ultricies sed, dolor. Cras elementum ultrices diam. Maecenas ligula massa, varius a, semper congue, euismod non, mi. Proin porttitor, orci nec nonummy molestie, enim est eleifend mi, non fermentum diam nisl sit amet erat. Duis semper. Duis arcu massa, scelerisque vitae, consequat in, pretium a, enim. Pellentesque congue. Ut in risus volutpat libero pharetra tempor. Cras vestibulum bibendum augue. Praesent egestas leo in pede. Praesent blandit odio eu enim. Pellentesque sed dui ut augue blandit sodales. Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia Curae; Aliquam nibh. Mauris ac mauris sed pede pellentesque fermentum. Maecenas adipiscing ante non diam sodales hendrerit.".to_string())
        ],
        files_inc: 3,
        files: vec![DropFile {
            id: 2 ,
            name: "file.txt".to_string() ,
            upload_secs: NOW ,
            chunks: vec![
                Chunk{len: 3, oid: 2001},
                Chunk{len: 4, oid: 2002},
            ]
        }],
    }).await ?;

    db.fs_set(101, 2001, Bytes::from_static(b"123")).await?;
    db.fs_set(101, 2002, Bytes::from_static(b"456!")).await?;

    // Wiki
    db.obj_set(
        201,
        OID_ENTITY_WIKI as u32,
        op::wiki::State {
            owner_id: 201,
            articles: {
                let mut m = BTreeMap::new();
                m.insert(
                    401,
                    op::wiki::Article {
                        article_id: 401,
                        slug: "foo".to_string(),
                        title: "Foo article".to_string(),
                        last_edit: NOW,
                    },
                );
                m.insert(
                    402,
                    op::wiki::Article {
                        article_id: 402,
                        slug: "bar".to_string(),
                        title: "Bar article".to_string(),
                        last_edit: 1785597192,
                    },
                );
                m
            },
        },
    )
    .await?;
    db.fs_set(201, 401 , Bytes::from_static(b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed non risus. Suspendisse lectus tortor, dignissim sit amet, adipiscing nec, ultricies sed, dolor. Cras elementum ultrices diam. Maecenas ligula massa, varius a, semper congue, euismod non, mi. Proin porttitor, orci nec nonummy molestie, enim est eleifend mi, non fermentum diam nisl sit amet erat. Duis semper. Duis arcu massa, scelerisque vitae, consequat in, pretium a, enim. Pellentesque congue. Ut in risus volutpat libero pharetra tempor. Cras vestibulum bibendum augue. Praesent egestas leo in pede. Praesent blandit odio eu enim. Pellentesque sed dui ut augue blandit sodales. Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia Curae; Aliquam nibh. Mauris ac mauris sed pede pellentesque fermentum. Maecenas adipiscing ante non diam sodales hendrerit.

Ut velit mauris, egestas sed, gravida nec, ornare ut, mi. Aenean ut orci vel massa suscipit pulvinar. Nulla sollicitudin. Fusce varius, ligula non tempus aliquam, nunc turpis ullamcorper nibh, in tempus sapien eros vitae ligula. Pellentesque rhoncus nunc et augue. Integer id felis. Curabitur aliquet pellentesque diam. Integer quis metus vitae elit lobortis egestas. Lorem ipsum dolor sit amet, consectetuer adipiscing elit. Morbi vel erat non mauris convallis vehicula. Nulla et sapien. Integer tortor tellus, aliquam faucibus, convallis id, congue eu, quam. Mauris ullamcorper felis vitae erat. Proin feugiat, augue non elementum posuere, metus purus iaculis lectus, et tristique ligula justo vitae magna.

liquam convallis sollicitudin purus. Praesent aliquam, enim at fermentum mollis, ligula massa adipiscing nisl, ac euismod nibh nisl eu lectus. Fusce vulputate sem at sapien. Vivamus leo. Aliquam euismod libero eu enim. Nulla nec felis sed leo placerat imperdiet. Aenean suscipit nulla in justo. Suspendisse cursus rutrum augue. Nulla tincidunt tincidunt mi. Curabitur iaculis, lorem vel rhoncus faucibus, felis magna fermentum augue, et ultricies lacus lorem varius purus. Curabitur eu amet..")).await?;
    db.fs_set(201, 402, Bytes::from_static(b"Hello World"))
        .await?;

    // Final init
    common.init(&db).await?;

    let common = Arc::new(common);
    init_sarah_data(&db, common.clone()).await;

    Ok((db, common))
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
            access: vec![
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
            ],
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
}
