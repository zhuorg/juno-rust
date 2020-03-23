use crate::{
	connection::{BaseConnection, Buffer, UnixSocketConnection},
	models::BaseMessage,
	protocol::BaseProtocol,
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
use serde_json::{Map, Value};
use std::collections::HashMap;

pub struct GothamModule {
	protocol: BaseProtocol,
	connection: Box<dyn BaseConnection>,
	requests: Arc<Mutex<HashMap<String, Sender<Value>>>>,
	functions: HashMap<String, fn(Value)>,
	hook_listeners: HashMap<String, Vec<fn(Value)>>,
	message_buffer: Vec<u8>,
	registered: bool,
}

impl GothamModule {
	pub fn default(socket_path: String) -> Self {
		GothamModule {
			protocol: BaseProtocol::default(),
			connection: Box::new(UnixSocketConnection::new(socket_path)),
			requests: Arc::new(Mutex::new(HashMap::new())),
			functions: HashMap::new(),
			hook_listeners: HashMap::new(),
			message_buffer: vec![],
			registered: false,
		}
	}

	pub fn new(protocol: BaseProtocol, connection: Box<dyn BaseConnection>) -> Self {
		GothamModule {
			protocol,
			connection,
			requests: Arc::new(Mutex::new(HashMap::new())),
			functions: HashMap::new(),
			hook_listeners: HashMap::new(),
			message_buffer: vec![],
			registered: false,
		}
	}

	pub async fn initialize(
		&mut self,
		module_id: String,
		version: String,
		dependencies: HashMap<String, String>,
	) {
		self.setup_connections().await;

		let request = self.protocol.initialize(module_id, version, dependencies);
		self.send_request(request)
			.await
			.await
			.unwrap_or(Value::Null);
		self.registered = true;
	}

	async fn setup_connections(&mut self) {
		self.connection.setup_connection().await;

		let data_receiver = self.connection.get_data_receiver();
		let requests = self.requests.clone();
		let protocol = BaseProtocol::from(&self.protocol);
		task::spawn(async {
			on_data_listener(data_receiver, requests, protocol).await;
		});
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

async fn on_data_listener(
	mut receiver: UnboundedReceiver<Buffer>,
	requests: Arc<Mutex<HashMap<String, Sender<Value>>>>,
	protocol: BaseProtocol,
) {
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
