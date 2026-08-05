use bytes::Bytes;
use n1_tool::{Chunk, Config, ConfigMemoryMutex, Result};

use crate::{
    OpServer,
    op::{
        self, OID_ENTITY_DROPBOX, TokenLevel,
        dropbox::DropFile,
        user::{Entity, Group, User},
    },
};

pub async fn init_dev() -> Result<OpServer<n1_tool::ConfigMemoryMutex>> {
    let mut server = op::init(ConfigMemoryMutex::new()).await?;

    // Set users
    let entities_map = server.entities.get_mut()?;
    entities_map.insert(
        101,
        Entity::User(User {
            uid: 101,
            name: "eve".to_string(),
            password: "56".to_string(),
            global: TokenLevel::Admin,
            groups_array: [
                (201, TokenLevel::Admin),
                (0, TokenLevel::None),
                (0, TokenLevel::None),
                (0, TokenLevel::None),
                (0, TokenLevel::None),
            ],
            groups_vec: Vec::with_capacity(0),
        }),
    );

    entities_map.insert(
        201,
        Entity::Group(Group {
            gid: 201,
            name: "world".to_string(),
            users: vec![(101, TokenLevel::Admin)],
        }),
    );

    // Set dropbox
    server.config.obj_store(101 , OID_ENTITY_DROPBOX, op::dropbox::State{
        texts_inc: 3  ,
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
                Chunk{len:3   , oid: 2001 } ,
                Chunk{len: 4, oid: 2002 } ,
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

    Ok(server)
}
