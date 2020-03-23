pub mod request_keys {
	pub const TYPE: &str = "type";
	pub const REQUEST_ID: &str = "requestId";
	pub const MODULE_ID: &str = "moduleId";
	pub const VERSION: &str = "version";
	pub const DEPENDENCIES: &str = "dependencies";
	pub const ERROR: &str = "error";
	pub const FUNCTION: &str = "function";
	pub const HOOK: &str = "hook";
	pub const ARGUMENTS: &str = "arguments";
	pub const DATA: &str = "data";
}

pub const CALL_FUNCTION_REQUEST_ID: &str = "call_function_internal";
pub const TRIGGER_HOOK_REQUEST_ID: &str = "trigger_hook_internal";
