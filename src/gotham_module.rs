use crate::{
	connection::{BaseConnection, Buffer, UnixSocketConnection},
	models::BaseMessage,
	protocol::BaseProtocol,
	utils,
};
use async_std::{
	prelude::*,
	sync::{Arc, Mutex},
	task,
};
use futures::channel::{
	mpsc::{UnboundedReceiver, UnboundedSender},
	oneshot::{channel, Receiver, Sender},
};
use futures_util::sink::SinkExt;
use serde_json::{Map, Value};
use std::collections::HashMap;

type SharableRequestList = Arc<Mutex<HashMap<String, Sender<Value>>>>;
type SharableFunctionList = Arc<Mutex<HashMap<String, fn(Map<String, Value>) -> Value>>>;
type SharableHookList = Arc<Mutex<HashMap<String, Vec<fn(Value)>>>>;

pub struct GothamModule {
	protocol: BaseProtocol,
	connection: Box<dyn BaseConnection>,
	requests: SharableRequestList,
	functions: SharableFunctionList,
	hook_listeners: SharableHookList,
	message_buffer: Vec<u8>,
	registered: bool,
}

impl GothamModule {
	pub fn default(socket_path: String) -> Self {
		GothamModule {
			protocol: BaseProtocol::default(),
			connection: Box::new(UnixSocketConnection::new(socket_path)),
			requests: Arc::new(Mutex::new(HashMap::new())),
			functions: Arc::new(Mutex::new(HashMap::new())),
			hook_listeners: Arc::new(Mutex::new(HashMap::new())),
			message_buffer: vec![],
			registered: false,
		}
	}

	pub fn new(protocol: BaseProtocol, connection: Box<dyn BaseConnection>) -> Self {
		GothamModule {
			protocol,
			connection,
			requests: Arc::new(Mutex::new(HashMap::new())),
			functions: Arc::new(Mutex::new(HashMap::new())),
			hook_listeners: Arc::new(Mutex::new(HashMap::new())),
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

	pub async fn declare_function(
		&mut self,
		fn_name: String,
		function: fn(Map<String, Value>) -> Value,
	) {
		self.functions
			.lock()
			.await
			.insert(fn_name.clone(), function);

		let request = self.protocol.declare_function(fn_name);
		self.send_request(request)
			.await
			.await
			.unwrap_or(Value::Null);
	}

	pub async fn call_function(&mut self, fn_name: String, args: Map<String, Value>) -> Value {
		let request = self.protocol.call_function(fn_name, args);
		self.send_request(request)
			.await
			.await
			.unwrap_or(Value::Null)
	}

	pub async fn register_hook(&mut self, hook: String, callback: fn(Value)) {
		let mut hook_listeners = self.hook_listeners.lock().await;
		if hook_listeners.contains_key(&hook) {
			hook_listeners.get_mut(&hook).unwrap().push(callback);
		} else {
			hook_listeners.insert(hook.clone(), vec![callback]);
		}
		drop(hook_listeners);

		let request = self.protocol.register_hook(hook);
		self.send_request(request)
			.await
			.await
			.unwrap_or(Value::Null);
	}

	pub async fn trigger_hook(&mut self, hook: String) {
		let request = self.protocol.trigger_hook(hook);
		self.send_request(request)
			.await
			.await
			.unwrap_or(Value::Null);
	}

	pub async fn close(&mut self) {
		self.connection.close_connection().await;
	}

	async fn setup_connections(&mut self) {
		self.connection.setup_connection().await;

		// Setup the multi-threaded read-write loop
		let data_receiver = self.connection.get_data_receiver();
		let write_sender = self.connection.clone_write_sender();
		let protocol = BaseProtocol::from(&self.protocol);
		let requests = self.requests.clone();
		let functions = self.functions.clone();
		let hook_listeners = self.hook_listeners.clone();

		// Run the read-write loop
		task::spawn(async {
			on_data_listener(data_receiver, protocol, requests, functions, hook_listeners, write_sender).await;
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
	protocol: BaseProtocol,
	requests: SharableRequestList,
	functions: SharableFunctionList,
	hook_listeners: SharableHookList,
	mut write_sender: UnboundedSender<Buffer>,
) {
	while let Some(data) = receiver.next().await {
		let message = protocol.decode(data.as_slice());
		let mut requests = requests.lock().await;
		let request_id = message.get_request_id().clone();

		if request_id == utils::CALL_FUNCTION_REQUEST_ID
			|| request_id == utils::TRIGGER_HOOK_REQUEST_ID
		{
			drop(requests);
			continue;
		}

		let value = match message {
			BaseMessage::FunctionCallResponse { data, .. } => data,
			BaseMessage::FunctionCallRequest { .. } => {
				let result = execute_function_call(message, &functions).await;
				let write_buffer = protocol.encode(&BaseMessage::FunctionCallResponse {
					request_id: request_id.clone(),
					data: result,
				});
				if let Err(err) = write_sender.send(write_buffer).await {
					println!("Error writing back result of function call: {}", err);
				}
				Value::Null
			}
			BaseMessage::TriggerHookResponse { .. } => execute_hook_triggered(message, &hook_listeners).await,
			_ => Value::Null,
		};

		if !requests.contains_key(&request_id) {
			drop(requests);
			continue;
		}
		if let Err(err) = requests.remove(&request_id).unwrap().send(value) {
			println!("Error sending response: {}", err);
		}
		drop(requests);
	}
}

async fn execute_function_call(
	message: BaseMessage,
	functions: &SharableFunctionList,
) -> Value {
	if let BaseMessage::FunctionCallRequest {
		function,
		arguments,
		..
	} = message
	{
		let functions = functions.lock().await;
		if !functions.contains_key(&function) {
			todo!("Wtf do I do now? Need to propogate errors. How do I do that?");
		}
		functions[&function](arguments)
	} else {
		panic!("Cannot execute function from a request that wasn't a FunctionCallRequest!");
	}
}

async fn execute_hook_triggered(
	message: BaseMessage,
	hook_listeners: &SharableHookList,
) -> Value {
	if let BaseMessage::TriggerHookRequest { hook, .. } = message {
		let hook_listeners = hook_listeners.lock().await;
		if !hook_listeners.contains_key(&hook) {
			todo!("Wtf do I do now? Need to propogate errors. How do I do that?");
		}
		for listener in &hook_listeners[&hook] {
			listener(Value::Null);
		}
	} else {
		panic!("Cannot execute function from a request that wasn't a FunctionCallRequest!");
	}
	Value::Null
}
