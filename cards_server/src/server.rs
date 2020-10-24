use crate::game::{Game, ThreadSafeGame};

use cards_protocol as proto;
use smol::{net, prelude::*};

use tracing::{info, instrument, warn, error, debug};

pub fn run(games: Vec<Game>) -> ! {
    smol::block_on(web_server(games))
}

#[instrument(skip(games))]
async fn web_server(games: Vec<Game>) -> ! {
    // let span = span!(Level::INFO, "web server");
    // let _enter = span.enter();
    let games: Vec<ThreadSafeGame> = games.iter().map(|x| x.thread_safe()).collect();
    let listener = match net::TcpListener::bind("0.0.0.0:25566").await {
        Ok(x) => x,
        Err(_) => net::TcpListener::bind("0.0.0.0:0").await.unwrap(),
    };
    info!(
        "Listening on port {}",
        listener.local_addr().unwrap().port()
    );
    let mut incoming = listener.incoming();
    let server = proto::ServerProtocol::new();
    while let Some(stream) = incoming.next().await {
        let uuid = server.connection(stream.unwrap()).await;
        smol::spawn(handle_connection(server.clone(), uuid, games.clone())).detach();
    }
    unreachable!()
}

#[instrument(skip(server, games))]
async fn handle_connection(
    server: proto::ServerProtocol,
    uuid: proto::Uuid,
    games: Vec<ThreadSafeGame>,
) -> () {
    let addr = server.peer_addr(&uuid).await.unwrap();
    // let span = span!(Level::INFO, format!("{} - {}", addr, uuid));
    // let _enter = span.enter();
    info!("Connected to {}", addr);
    loop {
        match server.recv(&uuid).await {
            Ok(req) => {
                info!("{:?}", req);
                match req {
                    proto::Request::Games => server
                        .send(
                            &uuid,
                            &proto::Reply::Games(
                                games
                                    .iter()
                                    .map(|g| (g.name().clone(), g.version().clone()))
                                    .collect(),
                            ),
                        )
                        .await
                        .unwrap(),
                }
            }
            Err(e) => {
                if let std::io::ErrorKind::NotConnected = e.kind() {
                    break;
                }
                info!("{:?}", e)
            }
        }
    }
    info!("Disconnected from {}", addr);
}
