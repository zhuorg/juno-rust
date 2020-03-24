use crate::{
	connection::Buffer,
	models::BaseMessage,
	protocol::base_protocol::BaseProtocol,
	utils::request_keys,
};
use serde_json::{from_slice, json, Result, Value};
use std::collections::HashMap;

pub fn default() -> BaseProtocol {
	BaseProtocol::JsonProtocol {
		module_id: String::default(),
	}
}

pub fn from(other: &BaseProtocol) -> BaseProtocol {
	match other {
		BaseProtocol::JsonProtocol { module_id } => BaseProtocol::JsonProtocol {
			module_id: module_id.clone(),
		},
		_ => panic!("BaseProtocol tried to decode a non-JsonProtocol as a JsonProtocol"),
	}
}

pub fn encode(protocol: &BaseProtocol, req: &BaseMessage) -> Buffer {
	match protocol {
		BaseProtocol::JsonProtocol { .. } => format!(
			"{}\n",
			match req {
				BaseMessage::RegisterModuleRequest {
					request_id,
					module_id,
					version,
					dependencies,
				} => json!({
					request_keys::REQUEST_ID: request_id,
					request_keys::TYPE: req.get_type(),
					request_keys::MODULE_ID: module_id,
					request_keys::VERSION: version,
					request_keys::DEPENDENCIES: dependencies,
				}),

				BaseMessage::RegisterModuleResponse { request_id } => json!({
					request_keys::REQUEST_ID: request_id,
					request_keys::TYPE: req.get_type(),
				}),

				BaseMessage::FunctionCallRequest {
					request_id,
					function,
					arguments,
				} => json!({
					request_keys::REQUEST_ID: request_id,
					request_keys::TYPE: req.get_type(),
					request_keys::FUNCTION: function,
					request_keys::ARGUMENTS: arguments,
				}),

				BaseMessage::FunctionCallResponse { request_id, data } => json!({
					request_keys::REQUEST_ID: request_id,
					request_keys::TYPE: req.get_type(),
					request_keys::DATA: data,
				}),

				BaseMessage::RegisterHookRequest { request_id, hook } => json!({
					request_keys::REQUEST_ID: request_id,
					request_keys::TYPE: req.get_type(),
					request_keys::HOOK: hook,
				}),

				BaseMessage::ListenHookResponse { request_id } => json!({
					request_keys::REQUEST_ID: request_id,
					request_keys::TYPE: req.get_type(),
				}),

				BaseMessage::TriggerHookRequest { request_id, hook } => json!({
					request_keys::REQUEST_ID: request_id,
					request_keys::TYPE: req.get_type(),
					request_keys::HOOK: hook,
				}),

				BaseMessage::TriggerHookResponse { request_id } => json!({
					request_keys::REQUEST_ID: request_id,
					request_keys::TYPE: req.get_type(),
				}),

				BaseMessage::DeclareFunctionRequest {
					request_id,
					function,
				} => json!({
					request_keys::REQUEST_ID: request_id,
					request_keys::TYPE: req.get_type(),
					request_keys::FUNCTION: function,
				}),

				BaseMessage::DeclareFunctionResponse {
					request_id,
					function,
				} => json!({
					request_keys::REQUEST_ID: request_id,
					request_keys::TYPE: req.get_type(),
					request_keys::FUNCTION: function,
				}),

				BaseMessage::Unknown { .. } => json!({
					request_keys::REQUEST_ID: "undefined",
					request_keys::TYPE: req.get_type(),
					request_keys::ERROR: 0
				}),

				BaseMessage::Error {
					request_id,
					error
				} => json!({
					request_keys::REQUEST_ID: request_id,
					request_keys::TYPE: req.get_type(),
					request_keys::ERROR: error
				}),
			}
			.to_string()
		)
		.as_bytes()
		.to_vec(),
		_ => panic!("BaseProtocol tried to decode a non-JsonProtocol as a JsonProtocol"),
	}
}

pub fn decode(protocol: &BaseProtocol, data: &[u8]) -> BaseMessage {
	match protocol {
		BaseProtocol::JsonProtocol { .. } => match decode_internal(data) {
			Some(msg) => msg,
			None => BaseMessage::Unknown {
				request_id: String::default(),
			},
		},
		_ => panic!("BaseProtocol tried to decode a non-JsonProtocol as a JsonProtocol"),
	}
}

fn decode_internal(data: &[u8]) -> Option<BaseMessage> {
	let result: Result<Value> = from_slice(data);
	if result.is_err() {
		return None;
	}
	let result = result.unwrap();

	let r#type = result[request_keys::TYPE].as_u64()?;

	if r#type == 1 {
		let request_id = result[request_keys::REQUEST_ID].as_str()?.to_string();
		let module_id = result[request_keys::MODULE_ID].as_str()?.to_string();
		let version = result[request_keys::VERSION].as_str()?.to_string();
		let dependencies_map = result[request_keys::DEPENDENCIES].as_object();

		let mut dependencies = HashMap::new();

		// If dependencies are not null, then populate the dependency_map
		if let Some(dependencies_map) = dependencies_map {
			for dependency in dependencies_map.keys() {
				if !dependencies_map[dependency].is_string() {
					return None;
				}
				let dependency_requirement = dependencies_map[dependency].as_str()?.to_string();
				dependencies.insert(dependency.clone(), dependency_requirement);
			}
		} else {
			return None;
		}

		Some(BaseMessage::RegisterModuleRequest {
			request_id,
			module_id,
			version,
			dependencies,
		})
	} else if r#type == 2 {
		let request_id = result[request_keys::REQUEST_ID].as_str()?.to_string();

		Some(BaseMessage::RegisterModuleResponse { request_id })
	} else if r#type == 3 {
		let request_id = result[request_keys::REQUEST_ID].as_str()?.to_string();
		let function = result[request_keys::FUNCTION].as_str()?.to_string();
		let arguments = result[request_keys::ARGUMENTS].as_object()?.clone();

		Some(BaseMessage::FunctionCallRequest {
			request_id,
			function,
			arguments,
		})
	} else if r#type == 4 {
		let request_id = result[request_keys::REQUEST_ID].as_str()?.to_string();
		let data = result[request_keys::DATA].clone();

		Some(BaseMessage::FunctionCallResponse { request_id, data })
	} else if r#type == 5 {
		let request_id = result[request_keys::REQUEST_ID].as_str()?.to_string();
		let hook = result[request_keys::HOOK].as_str()?.to_string();

		Some(BaseMessage::RegisterHookRequest { request_id, hook })
	} else if r#type == 6 {
		let request_id = result[request_keys::REQUEST_ID].as_str()?.to_string();

		Some(BaseMessage::ListenHookResponse { request_id })
	} else if r#type == 7 {
		let request_id = result[request_keys::REQUEST_ID].as_str()?.to_string();
		let hook = result[request_keys::HOOK].as_str()?.to_string();

		Some(BaseMessage::TriggerHookRequest { request_id, hook })
	} else if r#type == 8 {
		let request_id = result[request_keys::REQUEST_ID].as_str()?.to_string();

		Some(BaseMessage::TriggerHookResponse { request_id })
	} else if r#type == 9 {
		let request_id = result[request_keys::REQUEST_ID].as_str()?.to_string();
		let function = result[request_keys::FUNCTION].as_str()?.to_string();

		Some(BaseMessage::DeclareFunctionRequest {
			request_id,
			function,
		})
	} else if r#type == 10 {
		let request_id = result[request_keys::REQUEST_ID].as_str()?.to_string();
		let function = result[request_keys::FUNCTION].as_str()?.to_string();

		Some(BaseMessage::DeclareFunctionResponse {
			request_id,
			function,
		})
	} else {
		Some(BaseMessage::Unknown {
			request_id: String::default(),
		})
	}
}
