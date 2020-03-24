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

#[allow(dead_code)]
pub mod errors {
	pub const MALFORMED_REQUEST: u32 = 0;

	pub const INVALID_REQUEST_ID: u32 = 1;
	pub const UNKNOWN_REQUEST: u32 = 2;
	pub const UNREGISTERED_MODULE: u32 = 3;
	pub const UNKNOWN_MODULE: u32 = 4;
	pub const UNKNOWN_FUNCTION: u32 = 5;
	pub const INVALID_MODULE_ID: u32 = 6;
	pub const DUPLICATE_MODULE: u32 = 7;
}
