use std::{io, sync::Arc};

use n1_core::{
    init_dev,
    proto_http::{self, HTTPServer},
};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut key = [0u8; 64];
    for i in 0..key.len() {
        key[i] = i as u8;
    }

    let op = init_dev()
        .await
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err.atomic.msg))?;

    let server = Arc::new(HTTPServer { op, key });

    proto_http::run(server.clone()).await
}
