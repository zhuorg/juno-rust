use std::collections::HashMap;

trait BaseMessage {
	fn get_request_id(&self) -> &String;
	fn get_type(&self) -> u64;
}

pub struct RegisterModuleRequest {
	pub request_id: String,
	pub module_id: String,
	pub version: String,
	pub dependencies: HashMap<String, String>,
}

impl BaseMessage for RegisterModuleRequest {
	fn get_request_id(&self) -> &String {
		&self.request_id
	}

	fn get_type(&self) -> u64 {	1 }
}
/*
export interface DeclareFunctionRequest extends BaseMessage {
	function: string;
}

export interface FunctionCallRequest extends BaseMessage {
	function: string;
	arguments: {
		[type: string]: any
	};
}

export interface RegisterHookRequest extends BaseMessage {
	hook: string;
}

export interface TriggerHookRequest extends BaseMessage {
	hook: string;
}

export interface RegisterModuleResponse extends BaseMessage {
	test: string;
}

export interface FunctionCallResponse extends BaseMessage {
	data: any;
}

export interface ListenHookResponse extends BaseMessage {
}

export interface TriggerHookResponse extends BaseMessage {
	hook: string;
	data?: any;
}

export interface DeclareFunctionResponse extends BaseMessage {
	function: string;
}

export type GothamResponse =
	RegisterModuleResponse |
	ListenHookResponse |
	TriggerHookResponse |
	DeclareFunctionResponse |
	FunctionCallResponse;
export type GothamRequest =
	RegisterModuleRequest |
	DeclareFunctionRequest |
	FunctionCallRequest |
	RegisterHookRequest |
	TriggerHookRequest;
export type GothamMessage =
	RegisterModuleResponse |
	ListenHookResponse |
	TriggerHookResponse |
	DeclareFunctionResponse |
	FunctionCallResponse |
	RegisterModuleRequest |
	DeclareFunctionRequest |
	FunctionCallRequest |
	RegisterHookRequest |
	TriggerHookRequest;
*/
