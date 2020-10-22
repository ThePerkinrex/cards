use smol::net::{TcpStream, Shutdown};
use cards_protocol::{SendValue, Request};

fn main() {
    smol::block_on(async {
        let mut stream = TcpStream::connect("127.0.0.1:25566").await.unwrap();
        stream.send_val(&Request::Ready).await;
        stream.send_val(&Request::Games).await;
        stream.shutdown(Shutdown::Both).unwrap();
    });
}
