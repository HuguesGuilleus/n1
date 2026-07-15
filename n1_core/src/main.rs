use std::{io, sync::Arc};

use n1_core::{
    op::{self, OpServer},
    proto_http,
};
use n1_tool::ConfigMemoryMutex;

#[tokio::main]
async fn main() -> io::Result<()> {
    let server = Arc::new(OpServer {
        config: Arc::new(ConfigMemoryMutex::new()),
    });
    op::init(&server)
        .await
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err.msg))?;

    proto_http::run(server.clone()).await
}
