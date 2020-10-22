use crate::game::Game;

use smol::{net, prelude::*};
use cards_protocol as proto;


pub fn run(games: Vec<Game>) -> ! {
	smol::block_on(async {
		let listener = match net::TcpListener::bind("0.0.0.0:25566").await {
			Ok(x) => x,
			Err(_) => net::TcpListener::bind("0.0.0.0:0").await.unwrap()
		};
		println!("Listening on port {}", listener.local_addr().unwrap().port());
		let mut incoming = listener.incoming();
		while let Some(stream) = incoming.next().await {
			smol::spawn(handle(stream.unwrap())).detach();
		}
		unreachable!()
	})
}

async fn handle(stream: net::TcpStream) -> () {
	let addr = stream.peer_addr().unwrap();
	println!("Connected to: {}", addr);
	let mut server = proto::ServerProtocolStream::new(stream);
	loop {
		match server.recv().await {
			Ok(req) => {
				println!("{:?}", req);
			}
			Err(e) => println!("{:?}", e)
		}
		if server.should_close() {
			break
		}
	}
	println!("Closing connection to {}", addr);
}