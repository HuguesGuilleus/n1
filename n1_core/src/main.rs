use std::io;

use n1_core::proto_http;

#[tokio::main]
async fn main() -> io::Result<()> {
    proto_http::run().await
}
