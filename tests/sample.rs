use async_std::task;
use gotham::GothamModule;
use std::collections::HashMap;

#[test]
fn sample() {
	let mut module = GothamModule::default(String::from("../gotham.sock"));
	task::block_on(module.initialize("module1".to_string(), "1.0.0".to_string(), HashMap::new()));
	task::block_on(module.read_loop());
}
