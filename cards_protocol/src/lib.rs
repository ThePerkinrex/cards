
use smol::net::TcpStream;
use smol::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    Ready,
    Games,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Reply {
    Ready,
    Games(Vec<(String, String)>),
}

// TODO: Add wrapper for sending close signal when dropped

use async_trait::async_trait;

#[async_trait]
pub trait RecvValue<T>: AsyncRead + Unpin + Sized
where
    T: Sized,
    for<'a> T: Deserialize<'a>,
    T: 'static,
{
    async fn recv_val(&mut self) -> Result<T, std::io::Error> {
        let mut message_size = [0, 0, 0, 0];
        fill(&mut message_size, self)
            .await?;
		// println!("{:?}", message_size);
        let message_size: u32 = u32::from_le_bytes(message_size);
		println!("Message is {} bytes", message_size);
        let mut message = vec![0u8; message_size as usize];

        fill(&mut message, self)
            .await
			?;
		println!("{:?}", message);
        Ok(bincode::deserialize_from(message.as_slice()).expect("Malformed message"))
    }
}

async fn fill<R: AsyncRead + Unpin>(buf: &mut [u8], reader: &mut R) -> Result<(), std::io::Error> {
	(async{
		let mut reader = reader.take(buf.len() as u64);
		let mut intermidiate_buffer = vec![0u8; buf.len()];
		let mut read = 0;
		while read < buf.len()-1 {
			let read_bytes = reader.read(&mut intermidiate_buffer).await?;
			for i in read..read_bytes {
				buf[i] = intermidiate_buffer[i-read];
			}
			read += read_bytes;
	
			smol::future::yield_now().await;
		}
		Ok(())
	}).or(async {
		smol::Timer::after(std::time::Duration::from_secs(5)).await;
		Err(std::io::ErrorKind::TimedOut.into())
	}).await
}

impl<T> RecvValue<T> for TcpStream
where
    T: Sized,
    for<'a> T: Deserialize<'a>,
    T: 'static,
{
}

#[async_trait]
pub trait SendValue<T>: AsyncWrite + Unpin
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

// pub trait IntoStream: Sized {
//     fn encode(self, stream: &mut TcpStream) {
//         let message = self.encode_bytes();
//         let message_size = message.len() as u32;
//         assert!(message_size <= 1 << 24);
//         let message_size = message_size.to_le_bytes();
//         stream
//             .write_all(&message_size)
//             .expect("Error writing all message size bytes");
//         stream
//             .write_all(message)
//             .expect("Error writing all the message bytes");
//     }

//     fn encode_bytes<'a>(self) -> &'a [u8];
// }
