use crate::game::Game;

use smol::{net, prelude::*};
use cards_protocol as proto;
use proto::{RecvValue, SendValue};


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

async fn handle(mut stream: net::TcpStream) -> ! {
	println!("Connected to: {}", stream.peer_addr().unwrap());
	
	loop {
		match RecvValue::<proto::Request>::recv_val(&mut stream).await {
			Ok(req) => {
				println!("{:?}", req);
			}
			Err(e) => println!("{:?}", e)
		}
	}
}