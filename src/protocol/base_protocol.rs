use serde_json::{Map, Value};

use std::{
	collections::HashMap,
	time::{SystemTime, UNIX_EPOCH},
};

use crate::models::*;

pub trait BaseProtocol {
	fn generate_request_id(&self) -> String {
		let time = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.expect("Time went backwards. Wtf?")
			.as_nanos();
		format!("{}-{}", &self.get_module_id(), time)
	}

	fn get_module_id(&self) -> &String;
	fn set_module_id(&mut self, module_id: String);

	fn initialize(
		&mut self,
		module_id: String,
		version: String,
		dependencies: HashMap<String, String>,
	) -> BaseMessage {
		self.set_module_id(module_id);
		BaseMessage::RegisterModuleRequest {
			request_id: self.generate_request_id(),
			module_id: self.get_module_id().clone(),
			version,
			dependencies,
		}
	}

	fn register_hook(&self, hook: String) -> BaseMessage {
		BaseMessage::RegisterHookRequest {
			request_id: self.generate_request_id(),
			hook,
		}
	}

	fn trigger_hook(&self, hook: String) -> BaseMessage {
		BaseMessage::TriggerHookRequest {
			request_id: self.generate_request_id(),
			hook,
		}
	}

	fn declare_function(&self, function: String) -> BaseMessage {
		BaseMessage::DeclareFunctionRequest {
			request_id: self.generate_request_id(),
			function,
		}
	}

	fn call_function(&self, function: String, arguments: Map<String, Value>) -> BaseMessage {
		BaseMessage::FunctionCallRequest {
			request_id: self.generate_request_id(),
			function,
			arguments,
		}
	}

	fn encode(&self, req: &BaseMessage) -> Vec<u8>;
	fn decode(&self, data: &[u8]) -> BaseMessage;
}
