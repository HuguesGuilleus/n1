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

    let (db, mut  common) = init_dev()
        .await
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err.atomic.message))?;

common.lang = &n1_core::op::LANG_FR;

    let server = Arc::new(HTTPServer {
        db: Arc::new(db),
        common: Arc::new(common),
        key,
    });

    proto_http::run(server.clone()).await
}
