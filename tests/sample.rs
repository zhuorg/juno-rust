use async_std::task;
use gotham::{models::Value, GothamModule};
use std::collections::HashMap;

#[test]
fn sample() {
	let module1 = async {
		let mut module = GothamModule::from_unix_socket("../gotham.sock");
		module
			.initialize("module1", "1.0.0", HashMap::new())
			.await
			.unwrap();
		println!("initialized");
		module
			.declare_function("print_hello", |_| {
				println!("Hello");
				Value::Null
			})
			.await
			.unwrap();
		loop {
			task::sleep(std::time::Duration::from_millis(1000)).await;
		}
	};
	let module2 = async {
		let mut module = GothamModule::from_unix_socket("../gotham.sock");
		module
			.initialize("module2", "1.0.0", HashMap::new())
			.await
			.unwrap();
		module
			.call_function("module1.print_hello", HashMap::new())
			.await
			.unwrap();
		loop {
			task::sleep(std::time::Duration::from_millis(1000)).await;
		}
	};
	task::block_on(futures::future::join(module1, module2));
}
