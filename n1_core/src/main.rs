use std::{io, net::SocketAddr, sync::Arc};

use n1_core::{
    init_dev,
    proto_http::{HTTPServer, HTTPServerArc},
    Common,
};
use n1_tool::DBMemory;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut token_key = [0u8; 64];
    for i in 0..token_key.len() {
        token_key[i] = i as u8;
    }

    let db = DBMemory::new();
    let common = Arc::new(Common::new(&db).await.unwrap());

    init_dev(&db, common.clone())
        .await
        .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{:?}", err)))?;

    let server = HTTPServerArc(Arc::new(HTTPServer {
        db: db,
        common,
        token_key,
    }));

    let addr: SocketAddr = ([127, 0, 0, 1], 8000).into();
    let listener = TcpListener::bind(addr).await?;

    n1_tool::proto_http::run(listener, server).await
}
