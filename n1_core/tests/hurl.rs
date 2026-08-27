use hurl::{
    runner::{RunnerOptions, VariableSet, run},
    util::logger::{LoggerOptionsBuilder, Verbosity},
};
use hurl_core::input::Input;
use n1_core::{
    init_dev,
    proto_http::{HTTPServer, run_with_listener},
};
use std::{fs, net::SocketAddr, sync::Arc};
use tokio::{io, net::TcpListener, spawn};

#[tokio::test(flavor = "multi_thread")]
async fn hurl_tests() {
    // Server
    let mut key = [0u8; 64];
    for i in 0..key.len() {
        key[i] = i as u8;
    }
    let op = init_dev()
        .await
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err.atomic.message))
        .unwrap();
    let server = Arc::new(HTTPServer { op, key });

    let addr: SocketAddr = ([127, 0, 0, 1], 8000).into();
    let listener = TcpListener::bind(addr).await.unwrap();
    spawn(run_with_listener(listener, server));

    // Run HURL tests
    let mut paths: Vec<_> = std::fs::read_dir("tests")
        .unwrap()
        .map(|entry| format!("tests/{}", entry.unwrap().file_name().to_str().unwrap()))
        .filter(|path| path.ends_with(".hurl"))
        .collect();
    paths.sort();

    for path in paths {
        let content = fs::read_to_string(&path).unwrap();
        let result = run(
            &content,
            Some(&Input::new(&path)),
            &RunnerOptions::default(),
            &VariableSet::new(),
            &LoggerOptionsBuilder::new()
                .verbosity(Some(Verbosity::Verbose))
                .color(true)
                .build(),
        )
        .unwrap();
        if !result.errors().is_empty() {
            panic!()
        }
    }
}
