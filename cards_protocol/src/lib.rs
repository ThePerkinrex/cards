use serde::{Deserialize, Serialize};
use smol::net::TcpStream;
use smol::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    Close,
    Games,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Reply {
    Games(Vec<(String, String)>),
}

pub struct ServerProtocolStream {
    tcp: TcpStream,
    should_close: bool,
}

impl ServerProtocolStream {
    pub fn new(tcp: TcpStream) -> Self {
        Self {
            tcp,
            should_close: false,
        }
    }

    pub async fn recv(&mut self) -> Result<Request, std::io::Error> {
        if !self.should_close {
            match RecvValue::<Request>::recv_val(&mut self.tcp).await {
                Ok(Request::Close) => {
                    self.should_close = true;
                    println!("Should close");
                    Ok(Request::Close)
                }
                x => x,
            }
        } else {
            Err(std::io::ErrorKind::NotConnected.into())
        }
    }

    pub async fn send(&mut self, rep: &Reply) {
        if !self.should_close {
            self.tcp.send_val(rep).await
        }
    }

    pub fn should_close(&self) -> bool {
        self.should_close
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

    pub async fn send(&mut self, rep: &Request) {
        self.tcp.send_val(rep).await
    }
}

impl Drop for ClientProtocolStream {
    fn drop(&mut self) {
        println!("Dropping client");
        smol::block_on(self.send(&Request::Close))
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
    T: 'static,
{
    async fn send_val(&mut self, val: &T) {
        let message = bincode::serialize(val).unwrap();
        let message_size = message.len() as u32;
        assert!(message_size <= 1 << 24);
        println!("Message is {} bytes", message_size);
        println!("{:?}", message);
        let message_size = message_size.to_le_bytes();
        self.write_all(&message_size)
            .await
            .expect("Error writing all message size bytes");
        self.write_all(&message)
            .await
            .expect("Error writing all the message bytes");
    }
}

impl<T> SendValue<T> for TcpStream
where
    T: Send + Sync + Sized,
    T: Serialize,
    T: 'static,
{
}
