use async_std::{io::BufReader, os::unix::net::UnixStream, prelude::*};
use async_trait::async_trait;

use std::net::Shutdown;

use crate::connection::{BaseConnection, Buffer};

pub struct UnixSocketConnection {
	client: Option<UnixStream>,
	socket_path: String,
}

impl UnixSocketConnection {
	pub fn new(socket_path: String) -> Self {
		UnixSocketConnection {
			client: None,
			socket_path,
		}
	}
}

#[async_trait]
impl BaseConnection for UnixSocketConnection {
	async fn setup_connection(&mut self) {
		let result = UnixStream::connect(&self.socket_path).await;
		if let Err(err) = result {
			panic!("Error while connecting to socket: {}", err);
		}
		self.client = Some(result.unwrap());
	}

	async fn read_data(&self) -> Option<Buffer> {
		let reader = BufReader::new(self.client.as_ref().unwrap());
		let mut lines = reader.lines();

		if let Some(Ok(line)) = lines.next().await {
			return Some(line.into_bytes());
		}

		None
	}

	async fn close_connection(&mut self) {
		if let Some(client) = &self.client {
			if let Err(err) = client.shutdown(Shutdown::Both) {
				println!("Error while closing socket: {}", err);
			}
		}
	}

	async fn send(&mut self, buffer: Buffer) {
		if self.client.is_none() {
			println!("Cannot send data for a client that's not open");
			return;
		}
		let mut client = match &self.client {
			Some(client) => client,
			_ => panic!(),
		};
		if let Err(err) = client.write_all(&buffer).await {
			println!("Error while sending data to socket: {}", err);
		}
	}
}
