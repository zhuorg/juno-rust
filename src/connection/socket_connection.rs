use async_std::{
	io::BufReader,
	os::unix::net::UnixStream,
	prelude::{StreamExt, *},
	task,
};
use async_trait::async_trait;
use futures::{
	channel::{
		mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
		oneshot::{channel, Receiver, Sender},
	},
	future::{self, Either},
};
use futures_util::sink::SinkExt;

use std::net::Shutdown;

use crate::connection::{BaseConnection, Buffer};

pub struct UnixSocketConnection {
	socket_path: String,
	close_sender: Option<Sender<()>>,
	write_sender: Option<UnboundedSender<Vec<u8>>>,
}

impl UnixSocketConnection {
	pub fn new(socket_path: String) -> Self {
		UnixSocketConnection {
			socket_path,
			close_sender: None,
			write_sender: None
		}
	}
}

#[async_trait]
impl BaseConnection for UnixSocketConnection {
	fn connect_and_listen(&mut self, socket_path: String, data_sender: UnboundedSender<Vec<u8>>) {
		let (close_sender, close_listener) = channel::<()>();
		let (write_sender, write_listener) = unbounded::<Vec<u8>>();

		self.close_sender = Some(close_sender);
		self.write_sender = Some(write_sender);

		task::spawn(async {
			let mut sender = data_sender;
			let result = UnixStream::connect(socket_path).await;
			if let Err(err) = result {
				panic!("Error while connecting to socket: {}", err);
			}

			let client = result.unwrap();
			let reader = BufReader::new(client);
			let mut lines = reader.lines();

			let mut close_future = close_listener;
			let mut write_future = write_listener;

			while let Either::Left((Some(Ok(line)), next_close_future)) =
				future::select_all([lines.next(), close_future.receive(), write_future].into_iter()).await
			{
				close_future = next_close_future;
				let result = sender.send(line.as_bytes().to_vec()).await;
				if let Err(error) = result {
					println!("Error queing data to reader: {}", error);
				}
			}
			if let Err(err) = client.shutdown(Shutdown::Both) {
				println!("Error while closing socket: {}", err);
			}
		});
	}

	async fn close_connection(&mut self) {
		if let Some(sender) = &self.close_sender {
			if let Err(err) = sender.send(()) {
				println!("Error while sending command to close socket");
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
