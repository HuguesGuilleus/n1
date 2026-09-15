use bytes::Bytes;
use n1_tool::{AtomicError, Chunk, Config, ConfigMemoryMutex, Result};

use crate::{
    OpServer,
    op::{
        self, OID_ENTITY_DROPBOX, OID_ENTITY_WIKI, OID_GLOBAL_HOME, TokenItem,
        dropbox::DropFile,
        user::{Entity, Group, User},
    },
};

pub async fn init_dev() -> Result<OpServer<n1_tool::ConfigMemoryMutex>> {
    let mut server = OpServer::new(ConfigMemoryMutex::new());

    // Set user access
    let access: Vec<TokenItem> = vec![
        TokenItem {
            id: 101,
            app: OID_GLOBAL_HOME,
            can_write: true,
        },
        TokenItem {
            id: 101,
            app: OID_ENTITY_DROPBOX,
            can_write: true,
        },
        TokenItem {
            id: 201,
            app: OID_ENTITY_WIKI,
            can_write: true,
        },
    ];

    // Set users
    let entities_map = server.entities.get_mut().map_err(AtomicError::from)?;
    entities_map.insert(
        101,
        Entity::User(User {
            uid: 101,
            name: "eve".to_string(),
            password: "56".to_string(),
            is_admin: true,
            access,
            fs: op::fs::FsysState {
                dirs: vec![],
                dirs_increment: 0,
            },
        }),
    );

    entities_map.insert(
        201,
        Entity::Group(Group {
            gid: 201,
            name: "world".to_string(),
            users: vec![TokenItem {
                id: 101,
                app: OID_ENTITY_WIKI,
                can_write: true,
            }],
        }),
    );

    // Set dropbox
    server.config.obj_store(101 , OID_ENTITY_DROPBOX as u32 , op::dropbox::State{
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
            upload_secs: 1785597192,
            chunks: vec![
                Chunk{len: 3, oid: 2001},
                Chunk{len: 4, oid: 2002},
            ]
        }],
    }).await ?;
    server
        .config
        .fs_set(101, 2001, Bytes::from_static(b"123"))
        .await?;
    server
        .config
        .fs_set(101, 2002, Bytes::from_static(b"456!"))
        .await?;

    // Wiki
    server
        .config
        .obj_store(
            201,
            OID_ENTITY_WIKI as u32,
            op::wiki::State {
                eid: 201,
                shadow: 302,
                articles: vec![
                    op::wiki::Article {
                        oid: 401,
                        slug: "foo".to_string(),
                        title: "Foo article".to_string(),
                        last_edit: 1785597191,
                        content:r#"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed non risus. Suspendisse lectus tortor, dignissim sit amet, adipiscing nec, ultricies sed, dolor. Cras elementum ultrices diam. Maecenas ligula massa, varius a, semper congue, euismod non, mi. Proin porttitor, orci nec nonummy molestie, enim est eleifend mi, non fermentum diam nisl sit amet erat. Duis semper. Duis arcu massa, scelerisque vitae, consequat in, pretium a, enim. Pellentesque congue. Ut in risus volutpat libero pharetra tempor. Cras vestibulum bibendum augue. Praesent egestas leo in pede. Praesent blandit odio eu enim. Pellentesque sed dui ut augue blandit sodales. Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia Curae; Aliquam nibh. Mauris ac mauris sed pede pellentesque fermentum. Maecenas adipiscing ante non diam sodales hendrerit.

                        Ut velit mauris, egestas sed, gravida nec, ornare ut, mi. Aenean ut orci vel massa suscipit pulvinar. Nulla sollicitudin. Fusce varius, ligula non tempus aliquam, nunc turpis ullamcorper nibh, in tempus sapien eros vitae ligula. Pellentesque rhoncus nunc et augue. Integer id felis. Curabitur aliquet pellentesque diam. Integer quis metus vitae elit lobortis egestas. Lorem ipsum dolor sit amet, consectetuer adipiscing elit. Morbi vel erat non mauris convallis vehicula. Nulla et sapien. Integer tortor tellus, aliquam faucibus, convallis id, congue eu, quam. Mauris ullamcorper felis vitae erat. Proin feugiat, augue non elementum posuere, metus purus iaculis lectus, et tristique ligula justo vitae magna.

                        liquam convallis sollicitudin purus. Praesent aliquam, enim at fermentum mollis, ligula massa adipiscing nisl, ac euismod nibh nisl eu lectus. Fusce vulputate sem at sapien. Vivamus leo. Aliquam euismod libero eu enim. Nulla nec felis sed leo placerat imperdiet. Aenean suscipit nulla in justo. Suspendisse cursus rutrum augue. Nulla tincidunt tincidunt mi. Curabitur iaculis, lorem vel rhoncus faucibus, felis magna fermentum augue, et ultricies lacus lorem varius purus. Curabitur eu amet."#.to_string() ,
                    },
                    op::wiki::Article {
                        oid: 402,
                        slug: "bar".to_string(),
                        title: "Bar article".to_string(),
                        last_edit: 1785597192,
                        content: "bar article content...".to_string() ,
                     },
                ],
            },
        )
        .await?;
    server.config.fs_set(201, 401 , Bytes::from_static(b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed non risus. Suspendisse lectus tortor, dignissim sit amet, adipiscing nec, ultricies sed, dolor.\nCras elementum ultrices diam. Maecenas ligula massa, varius a, semper congue, euismod non, mi. Proin porttitor, orci nec nonummy molestie, enim est eleifend mi, non fermentum diam nisl sit amet erat. Duis semper. Duis arcu massa, scelerisque vitae, consequat in, pretium a, enim.\nPellentesque congue.\n Ut in risus volutpat libero pharetra tempor. Cras vestibulum bibendum augue. Praesent egestas leo in pede. Praesent blandit odio eu enim. Pellentesque sed dui ut augue blandit sodales. Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia Curae; Aliquam nibh. Mauris ac mauris sed pede pellentesque fermentum. Maecenas adipiscing ante non diam sodales hendrerit.")).await?;
    server
        .config
        .fs_set(201, 402, Bytes::from_static(b"Hello World"))
        .await?;

    // Final init
    server.init().await
}
