extern crate async_std;
extern crate futures;
extern crate futures_util;
extern crate serde_json;

pub use serde_json::json;

use crate::{
	connection::{BaseConnection, Buffer, InetSocketConnection},
	models::{BaseMessage, Value},
	protocol::BaseProtocol,
	utils::{self, Error, Result},
};

#[cfg(target_family = "unix")]
use crate::connection::UnixSocketConnection;

use async_std::{
	prelude::*,
	sync::{Arc, Mutex},
	task,
};
use futures::channel::{
	mpsc::{UnboundedReceiver, UnboundedSender},
	oneshot::{channel, Sender},
};
use futures_util::sink::SinkExt;
use std::{
	collections::HashMap,
	net::{AddrParseError, SocketAddr},
};

type ArcRequestList = Arc<Mutex<HashMap<String, Sender<Result<Value>>>>>;
type ArcFunctionList = Arc<Mutex<HashMap<String, fn(HashMap<String, Value>) -> Value>>>;
type ArcHookListenerList = Arc<Mutex<HashMap<String, Vec<fn(Value)>>>>;

pub struct JunoModule {
	protocol: BaseProtocol,
	connection: Box<dyn BaseConnection>,
	requests: ArcRequestList,
	functions: ArcFunctionList,
	hook_listeners: ArcHookListenerList,
	message_buffer: Buffer,
	registered: bool,
}

impl JunoModule {
	pub fn default(connection_path: &str) -> Self {
		let is_ip: std::result::Result<SocketAddr, AddrParseError> =
			connection_path.to_string().parse();
		if let Ok(ip) = is_ip {
			Self::from_inet_socket(&format!("{}", ip.ip()), ip.port())
		} else {
			Self::from_unix_socket(connection_path)
		}
	}

	#[cfg(target_family = "windows")]
	pub fn from_unix_socket(_: &str) -> Self {
		panic!("Unix sockets are not supported on windows");
	}

	#[cfg(target_family = "unix")]
	pub fn from_unix_socket(socket_path: &str) -> Self {
		JunoModule {
			protocol: BaseProtocol::default(),
			connection: Box::new(UnixSocketConnection::new(socket_path.to_string())),
			requests: Arc::new(Mutex::new(HashMap::new())),
			functions: Arc::new(Mutex::new(HashMap::new())),
			hook_listeners: Arc::new(Mutex::new(HashMap::new())),
			message_buffer: vec![],
			registered: false,
		}
	}

	pub fn from_inet_socket(host: &str, port: u16) -> Self {
		JunoModule {
			protocol: BaseProtocol::default(),
			connection: Box::new(InetSocketConnection::new(format!("{}:{}", host, port))),
			requests: Arc::new(Mutex::new(HashMap::new())),
			functions: Arc::new(Mutex::new(HashMap::new())),
			hook_listeners: Arc::new(Mutex::new(HashMap::new())),
			message_buffer: vec![],
			registered: false,
		}
	}

	pub fn new(protocol: BaseProtocol, connection: Box<dyn BaseConnection>) -> Self {
		JunoModule {
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
		module_id: &str,
		version: &str,
		dependencies: HashMap<String, String>,
	) -> Result<()> {
		self.setup_connections().await?;

		let request =
			self.protocol
				.initialize(String::from(module_id), String::from(version), dependencies);
		self.send_request(request).await?;

		self.registered = true;
		Ok(())
	}

	pub async fn declare_function(
		&mut self,
		fn_name: &str,
		function: fn(HashMap<String, Value>) -> Value,
	) -> Result<()> {
		let fn_name = fn_name.to_string();
		self.functions
			.lock()
			.await
			.insert(fn_name.clone(), function);

		let request = self.protocol.declare_function(fn_name);
		self.send_request(request).await?;
		Ok(())
	}

	pub async fn call_function(
		&mut self,
		fn_name: &str,
		args: HashMap<String, Value>,
	) -> Result<Value> {
		let fn_name = fn_name.to_string();
		self.ensure_registered()?;
		let request = self.protocol.call_function(fn_name, args);
		self.send_request(request).await
	}

	pub async fn register_hook(&mut self, hook: &str, callback: fn(Value)) -> Result<()> {
		let hook = hook.to_string();
		self.ensure_registered()?;
		let mut hook_listeners = self.hook_listeners.lock().await;
		if hook_listeners.contains_key(&hook) {
			hook_listeners.get_mut(&hook).unwrap().push(callback);
		} else {
			hook_listeners.insert(hook.clone(), vec![callback]);
		}
		drop(hook_listeners);

		let request = self.protocol.register_hook(hook);
		self.send_request(request).await?;
		Ok(())
	}

	pub async fn trigger_hook(&mut self, hook: &str) -> Result<()> {
		let hook = hook.to_string();
		let request = self.protocol.trigger_hook(hook);
		self.send_request(request).await?;
		Ok(())
	}

	pub async fn close(&mut self) {
		self.connection.close_connection().await;
	}

	fn ensure_registered(&self) -> Result<()> {
		if !self.registered {
			return Err(Error::Internal(String::from(
				"Module not registered. Did you .await the call to initialize?",
			)));
		}
		Ok(())
	}

	async fn setup_connections(&mut self) -> Result<()> {
		self.connection.setup_connection().await?;

		// Setup the multi-threaded read-write loop
		let data_receiver = self.connection.get_data_receiver();
		let write_sender = self.connection.clone_write_sender();
		let protocol = BaseProtocol::from(&self.protocol);
		let requests = self.requests.clone();
		let functions = self.functions.clone();
		let hook_listeners = self.hook_listeners.clone();

		// Run the read-write loop
		task::spawn(async {
			on_data_listener(
				data_receiver,
				protocol,
				requests,
				functions,
				hook_listeners,
				write_sender,
			)
			.await;
		});

		Ok(())
	}

	async fn send_request(&mut self, request: BaseMessage) -> Result<Value> {
		if let BaseMessage::RegisterModuleRequest { .. } = request {
			if self.registered {
				let (sender, receiver) = channel::<Result<Value>>();
				sender.send(Ok(Value::Null)).unwrap();

				return match receiver.await {
					Ok(value) => value,
					Err(_) => Err(Error::Internal(String::from(
						"Request sender was dropped before data could be retrieved",
					))),
				};
			}
		}

		let request_type = request.get_type();
		let request_id = request.get_request_id().clone();
		let mut encoded = self.protocol.encode(request);
		if self.registered || request_type == 1 {
			self.connection.send(encoded).await;
		} else {
			self.message_buffer.append(&mut encoded);
		}

		let (sender, receiver) = channel::<Result<Value>>();

		self.requests.lock().await.insert(request_id, sender);

		match receiver.await {
			Ok(value) => value,
			Err(_) => Err(Error::Internal(String::from(
				"Request sender was dropped before data could be retrieved",
			))),
		}
	}
}

async fn on_data_listener(
	mut receiver: UnboundedReceiver<Buffer>,
	protocol: BaseProtocol,
	requests: ArcRequestList,
	functions: ArcFunctionList,
	hook_listeners: ArcHookListenerList,
	mut write_sender: UnboundedSender<Buffer>,
) {
	while let Some(data) = receiver.next().await {
		let message = protocol.decode(data.as_slice());
		let mut requests = requests.lock().await;
		let request_id = message.get_request_id().clone();

		let value = match message {
			BaseMessage::FunctionCallResponse { data, .. } => Ok(data),
			BaseMessage::FunctionCallRequest { .. } => {
				let result = execute_function_call(message, &functions).await;
				let write_buffer = match result {
					Ok(value) => protocol.encode(BaseMessage::FunctionCallResponse {
						request_id: request_id.clone(),
						data: value,
					}),
					Err(error) => protocol.encode(BaseMessage::Error {
						request_id: request_id.clone(),
						error: match error {
							Error::Internal(_) => 0,
							Error::FromJuno(error_code) => error_code,
						},
					}),
				};
				if let Err(err) = write_sender.send(write_buffer).await {
					println!("Error writing back result of function call: {}", err);
				}
				Ok(Value::Null)
			}
			BaseMessage::TriggerHookRequest { .. } => {
				execute_hook_triggered(message, &hook_listeners).await
			}
			BaseMessage::Error { error, .. } => Err(Error::FromJuno(error)),
			_ => Ok(Value::Null),
		};

		if !requests.contains_key(&request_id) {
			drop(requests);
			continue;
		}
		if requests.remove(&request_id).unwrap().send(value).is_err() {
			println!("Error sending response of requestId: {}", &request_id);
		}
		drop(requests);
	}
}

async fn execute_function_call(message: BaseMessage, functions: &ArcFunctionList) -> Result<Value> {
	if let BaseMessage::FunctionCallRequest {
		function,
		arguments,
		..
	} = message
	{
		let functions = functions.lock().await;
		if !functions.contains_key(&function) {
			return Err(Error::FromJuno(utils::errors::UNKNOWN_FUNCTION));
		}
		Ok(functions[&function](arguments))
	} else {
		panic!("Cannot execute function from a request that wasn't a FunctionCallRequest!");
	}
}

async fn execute_hook_triggered(
	message: BaseMessage,
	hook_listeners: &ArcHookListenerList,
) -> Result<Value> {
	if let BaseMessage::TriggerHookRequest { hook, .. } = message {
		let hook_listeners = hook_listeners.lock().await;
		if !hook_listeners.contains_key(&hook) {
			todo!("Wtf do I do now? Need to propogate errors. How do I do that?");
		}
		for listener in &hook_listeners[&hook] {
			listener(Value::Null);
		}
	} else {
		panic!("Cannot execute hook from a request that wasn't a TriggerHookRequest!");
	}
	Ok(Value::Null)
}
