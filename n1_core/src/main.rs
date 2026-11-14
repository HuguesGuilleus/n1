use std::{io, net::SocketAddr, sync::Arc};

use n1_core::{
    init_dev,
    proto_http::{self, HTTPServer},
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut token_key = [0u8; 64];
    for i in 0..token_key.len() {
        token_key[i] = i as u8;
    }

    let (db, common) = init_dev()
        .await
        .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{:?}", err)))?;

    let server = Arc::new(HTTPServer {
        db: Arc::new(db),
        common,
        token_key,
    });

    let addr: SocketAddr = ([127, 0, 0, 1], 8000).into();
    let listener = TcpListener::bind(addr).await?;
    proto_http::run(listener, server.clone()).await
}
