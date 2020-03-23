use crate::{
	connection::{BaseConnection, Buffer, UnixSocketConnection},
	models::BaseMessage,
	protocol::{BaseProtocol, JsonProtocol},
};
use async_std::{
	prelude::*,
	sync::{Arc, Mutex},
	task,
};
use futures::channel::{
	mpsc::UnboundedReceiver,
	oneshot::{channel, Receiver, Sender},
};
use serde_json::Value;
use std::collections::HashMap;

pub struct GothamModule {
	protocol: Box<dyn BaseProtocol>,
	connection: Box<dyn BaseConnection>,
	registered: bool,
	message_buffer: Vec<u8>,
	requests: Arc<Mutex<HashMap<String, Sender<Value>>>>,
}

async fn on_data_listener(
	mut receiver: UnboundedReceiver<Buffer>,
	requests: Arc<Mutex<HashMap<String, Sender<Value>>>>,
) {
	let protocol = JsonProtocol::default();
	while let Some(data) = receiver.next().await {
		let message = protocol.decode(data.as_slice());
		let mut requests = requests.lock().await;
		let request_id = message.get_request_id();

		let value = Value::Null;

		if !requests.contains_key(request_id) {
			drop(requests);
			continue;
		}
		if let Err(err) = requests.remove(request_id).unwrap().send(value) {
			println!("Error sending response: {}", err);
		}
		drop(requests);
	}
}

impl GothamModule {
	pub fn default(socket_path: String) -> Self {
		GothamModule {
			protocol: Box::new(JsonProtocol::default()),
			connection: Box::new(UnixSocketConnection::new(socket_path)),
			registered: false,
			message_buffer: vec![],
			requests: Arc::new(Mutex::new(HashMap::new())),
		}
	}

	pub fn new(protocol: Box<dyn BaseProtocol>, connection: Box<dyn BaseConnection>) -> Self {
		GothamModule {
			protocol,
			connection,
			registered: false,
			message_buffer: vec![],
			requests: Arc::new(Mutex::new(HashMap::new())),
		}
	}

	pub async fn initialize(
		&mut self,
		module_id: String,
		version: String,
		dependencies: HashMap<String, String>,
	) -> Receiver<Value> {
		self.connection.setup_connection().await;
		let data_receiver = self.connection.get_data_receiver();
		let requests = self.requests.clone();
		task::spawn(async {
			on_data_listener(data_receiver, requests).await;
		});

		let initialze_request = self.protocol.initialize(module_id, version, dependencies);
		self.send_request(initialze_request).await
	}

	async fn send_request(&mut self, request: BaseMessage) -> Receiver<Value> {
		if let BaseMessage::RegisterModuleRequest { .. } = request {
			if self.registered {
				let (sender, receiver) = channel::<Value>();
				sender.send(Value::Null).unwrap();

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
			.lock()
			.await
			.insert(request.get_request_id().clone(), sender);

		receiver
	}
}
