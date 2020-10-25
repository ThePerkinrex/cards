use smol::net::TcpStream;
use cards_protocol::{Request, ClientProtocolStream};
use cards_subscriber::{ApplyTo, Subscriber, TargetKind, Filter};
use tracing::{Instrument, info};

fn main() {
    tracing::subscriber::set_global_default(Subscriber::new(
        "logs/client",
        &[],
        true,
    )
    .filter(Filter::new(Some(tracing::Level::INFO), Some(TargetKind::Target("cards_protocol")), ApplyTo::Stdout))
    ).unwrap();
    smol::block_on(async {
        let stream = TcpStream::connect("127.0.0.1:25566").await.unwrap();
        let mut client = ClientProtocolStream::new(stream);
        client.send(Request::Games).await.unwrap();
        info!("{:?}", client.recv().await);
    }.instrument(tracing::info_span!("client")));
}
