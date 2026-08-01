use std::{io, sync::Arc};

use n1_core::{
    init_dev,
    op::{self},
    proto_http::{self, HTTPServer},
};
use n1_tool::ConfigMemoryMutex;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut key = [0u8; 64];
    for i in 0..key.len() {
        key[i] = i as u8;
    }

    let mut op = op::init(ConfigMemoryMutex::new())
        .await
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err.msg))?;

    init_dev(&mut op).await.unwrap();

    let server = Arc::new(HTTPServer { op, key });

    proto_http::run(server.clone()).await
}
