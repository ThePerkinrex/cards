use smol::net::{TcpStream, Shutdown};
use cards_protocol::{Request, ClientProtocolStream};

fn main() {
    smol::block_on(async {
        let stream = TcpStream::connect("127.0.0.1:25566").await.unwrap();
        let mut client = ClientProtocolStream::new(stream);
        client.send(&Request::Games).await;
    });
}
