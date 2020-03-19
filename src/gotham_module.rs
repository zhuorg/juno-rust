use crate::{
	connection::{BaseConnection, Buffer, UnixSocketConnection},
	models::BaseMessage,
	protocol::{BaseProtocol, JsonProtocol},
};
use futures::channel::oneshot::{channel, Receiver, Sender};
use serde_json::Value;
use std::collections::HashMap;

#[allow(dead_code)]
pub struct GothamModule {
	protocol: Box<dyn BaseProtocol>,
	connection: Box<dyn BaseConnection>,
	registered: bool,
	message_buffer: Vec<u8>,
	requests: HashMap<String, Sender<Value>>,
}

impl GothamModule {
	pub fn default(socket_path: String) -> Self {
		GothamModule {
			protocol: Box::new(JsonProtocol::default()),
			connection: Box::new(UnixSocketConnection::new(socket_path)),
			registered: false,
			message_buffer: vec![],
			requests: HashMap::new(),
		}
	}

	pub fn new(protocol: Box<dyn BaseProtocol>, connection: Box<dyn BaseConnection>) -> Self {
		GothamModule {
			protocol,
			connection,
			registered: false,
			message_buffer: vec![],
			requests: HashMap::new(),
		}
	}

	pub async fn initialize(
		&mut self,
		module_id: String,
		version: String,
		dependencies: HashMap<String, String>,
	) -> Receiver<Value> {
		self.connection.setup_connection().await;
		let initialze_request = self.protocol.initialize(module_id, version, dependencies);
		self.send_request(initialze_request).await
	}

	pub async fn read_loop(&self) {
		while let Some(buffer) = &self.connection.read_data().await {
			self.on_data_handler(buffer.clone());
		}
	}

	async fn read_loop_until(&self, request_id: String) {}

	async fn send_request(&mut self, request: BaseMessage) -> Receiver<Value> {
		if let BaseMessage::RegisterModuleRequest { .. } = request {
			if self.registered {
				let (sender, receiver) = channel::<Value>();
				if let Err(err) = sender.send(Value::Null) {
					println!("Error sending to oneshot mpsc channel: {}", err);
				}

				return receiver;
			}
		}

		let mut encoded = self.protocol.encode(&request);
		if self.registered || request.get_type() == 1 {
			self.connection.send(encoded).await;
		} else {
			self.message_buffer.append(&mut encoded);
		}

		let (sender, receiver) = channel::<Value>();

		self.requests
			.insert(request.get_request_id().clone(), sender);

		receiver
	}

	fn on_data_handler(&self, data: Buffer) {
		let message = self.protocol.decode(data.as_slice());
	}
}
