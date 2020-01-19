use async_std::task;
use async_std::prelude::*;
use async_std::os::unix::net::UnixStream;

use std::net::Shutdown;

use crate::connection::{Buffer, OnDataHandler, BaseConnection};

pub struct UnixSocketConnection {
	on_data_handler: Option<OnDataHandler>,
	client: Option<UnixStream>,
	socket_path: String
}

impl BaseConnection for UnixSocketConnection {
	fn setup_connection(&mut self) {
		let result = task::block_on(UnixStream::connect(&self.socket_path));
		if let Err(err) = result {
			panic!("Error while connecting to socket: {}", err);
		}
		self.client = Some(result.unwrap());
	}

	fn close_connection(&mut self) {
		if let Some(client) = &self.client {
			if let Err(err) = client.shutdown(Shutdown::Both) {
				println!("Error while closing socket: {}", err);
			}
		}
	}

	fn send(&mut self, buffer: Buffer) {
		if let None = &self.client {
			println!("Cannot send data for a client that's not open");
			return;
		}
			let mut client = match &self.client {
				Some(client) => client,
				_ => panic!()
			};
			if let Err(err) = task::block_on(client.write_all(&buffer.data)) {
				println!("Error while sending data to socket: {}", err);
			}
	}

	fn set_on_data_listener(&mut self, on_data_handler: OnDataHandler) {
		self.on_data_handler = Some(on_data_handler);
	}

	fn on_data(&self, data: Buffer) {
		if let Some(handler) = &self.on_data_handler {
			(handler.function)(data);
		}
	}
}
