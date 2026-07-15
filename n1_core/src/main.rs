use std::{
    io,
    sync::{Arc, atomic::AtomicI64},
};

use n1_core::{
    op::{self, OpServer},
    proto_http,
};
use n1_tool::ConfigMemoryMutex;

#[tokio::main]
async fn main() -> io::Result<()> {
    let server = Arc::new(OpServer {
        config: Arc::new(ConfigMemoryMutex::new()),
        nb: AtomicI64::new(14),
    });
    op::init(&server)
        .await
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err.msg))?;

    proto_http::run(server.clone()).await
}
