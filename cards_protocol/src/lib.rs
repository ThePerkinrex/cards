use serde::{Deserialize, Serialize};

use smol::lock::RwLock;
use smol::net::TcpStream;
use smol::prelude::*;

use std::{collections::HashMap, sync::Arc};

pub use uuid::Uuid;

use tracing::{trace, instrument};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    Games,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
enum ServerRequest {
    Close,
    Message(Request)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Reply {
    Games(Vec<(String, String)>),
}

pub struct ServerProtocol {
    streams: Arc<RwLock<HashMap<uuid::Uuid, Arc<RwLock<TcpStream>>>>>,
}

impl ServerProtocol {
    pub fn new() -> Self {
        Self {
            streams: Default::default(),
        }
    }

    async fn get(&self, uuid: &Uuid) -> Option<Arc<RwLock<TcpStream>>> {
        self.streams.read().await.get(uuid).map(|x| x.clone())
    }

    async fn remove(&self, uuid: &Uuid) -> bool {
        self.streams.write().await.remove(uuid).is_some()
    }

    pub async fn connection(&self, tcp: TcpStream) -> Uuid {
        let mut uuid = Uuid::new_v4();
        while self.streams.read().await.contains_key(&uuid) {
            uuid = Uuid::new_v4();
        }
        self.streams.write().await.insert(uuid.clone(), Arc::new(RwLock::new(tcp)));
        uuid
    }

    pub async fn recv(&self, uuid: &Uuid) -> Result<Request, std::io::Error> {
        match self.get(uuid).await.clone() {
            Some(v) => {
                let mut should_close = false;
                let r = match v.write().await.recv_val().await {
                    Ok(ServerRequest::Close) => {
                        should_close = true;
                        Err(std::io::ErrorKind::NotConnected.into())
                    }
                    Ok(ServerRequest::Message(x)) => Ok(x),
                    Err(e) => Err(e),
                };

                if should_close {
                    self.remove(uuid).await;
                }
                r
            }
            None => Err(std::io::ErrorKind::NotConnected.into()),
        }
    }

    pub async fn send(&self, uuid: &Uuid, reply: &Reply) -> Result<(), std::io::Error> {
        match self.get(uuid).await.clone() {
            Some(v) => v.write().await.send_val(reply).await,
            None => Err(std::io::ErrorKind::NotConnected.into()),
        }
    }

    pub async fn peer_addr(&self, uuid: &Uuid) -> std::io::Result<std::net::SocketAddr> {
        match self.get(uuid).await.clone() {
            Some(v) => v.read().await.peer_addr(),
            None => Err(std::io::ErrorKind::NotConnected.into()),
        }
    }
}

impl Clone for ServerProtocol {
    fn clone(&self) -> Self {
        Self {
            streams: self.streams.clone(),
        }
    }
}

pub struct ClientProtocolStream {
    tcp: TcpStream,
}

impl ClientProtocolStream {
    pub fn new(tcp: TcpStream) -> Self {
        Self { tcp }
    }

    pub async fn recv(&mut self) -> Result<Reply, std::io::Error> {
        self.tcp.recv_val().await
    }

    pub async fn send(&mut self, req: Request) -> Result<(), std::io::Error> {
        self.tcp.send_val(&ServerRequest::Message(req)).await
    }
}

impl Drop for ClientProtocolStream {
    #[instrument(skip(self), name = "drop protocol client")]
    fn drop(&mut self) {
        trace!("Dropping client");
        if let Ok(()) = smol::block_on(self.tcp.send_val(&ServerRequest::Close)) {}
    }
}

use async_trait::async_trait;

#[async_trait]
trait RecvValue<T>: AsyncRead + Unpin + Sized
where
    T: Sized,
    for<'a> T: Deserialize<'a>,
    T: 'static,
{
    async fn recv_val(&mut self) -> Result<T, std::io::Error> {
        let mut message_size = [0, 0, 0, 0];
        fill(&mut message_size, self).await?;
        // println!("{:?}", message_size);
        let message_size: u32 = u32::from_le_bytes(message_size);
        // println!("Message is {} bytes", message_size);
        let mut message = vec![0u8; message_size as usize];

        fill(&mut message, self).await?;
        // println!("{:?}", message);
        Ok(bincode::deserialize_from(message.as_slice()).expect("Malformed message"))
    }
}

async fn fill<R: AsyncRead + Unpin>(buf: &mut [u8], reader: &mut R) -> Result<(), std::io::Error> {
    (async {
        let mut reader = reader.take(buf.len() as u64);
        let mut intermidiate_buffer = vec![0u8; buf.len()];
        let mut read = 0;
        while read < buf.len() - 1 {
            let read_bytes = reader.read(&mut intermidiate_buffer).await?;
            for i in read..read_bytes {
                buf[i] = intermidiate_buffer[i - read];
            }
            read += read_bytes;

            smol::future::yield_now().await;
        }
        Ok(())
    })
    .or(async {
        smol::Timer::after(std::time::Duration::from_secs(5)).await;
        Err(std::io::ErrorKind::TimedOut.into())
    })
    .await
}

impl<T> RecvValue<T> for TcpStream
where
    T: Sized,
    for<'a> T: Deserialize<'a>,
    T: 'static,
{
}

#[async_trait]
trait SendValue<T>: AsyncWrite + Unpin
where
    T: Send + Sync + Sized,
    T: Serialize,
    T: std::fmt::Debug,
    T: 'static,
{
    #[instrument(skip(self), name = "proto_send")]
    async fn send_val(&mut self, val: &T) -> Result<(), std::io::Error> {
        let message = bincode::serialize(val).unwrap();
        let message_size = message.len() as u32;
        assert!(message_size <= 1 << 24);
        trace!("Message is {} bytes", message_size);
        trace!("{:?}", message);
        let message_size = message_size.to_le_bytes();
        self.write_all(&message_size).await?;
        self.write_all(&message).await?;
        Ok(())
    }
}

impl<T> SendValue<T> for TcpStream
where
    T: Send + Sync + Sized,
    T: Serialize,
    T: std::fmt::Debug,
    T: 'static,
{
}
