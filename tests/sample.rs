#[test]
fn sample() {
	return;
	/*
	let module1 = async {
		let mut module = GothamModule::default(String::from("../gotham.sock"));
		module
			.initialize("module1".to_string(), "1.0.0".to_string(), HashMap::new())
			.await
			.unwrap();
		println!("initialized");
		module
			.declare_function("print_hello".to_string(), |_| {
				println!("Hello");
				serde_json::Value::Null
			})
			.await
			.unwrap();
		loop {
			task::sleep(std::time::Duration::from_millis(1000)).await;
		}
	};
	let module2 = async {
		let mut module = GothamModule::default(String::from("../gotham.sock"));
		module
			.initialize("module2".to_string(), "1.0.0".to_string(), HashMap::new())
			.await
			.unwrap();
		module
			.call_function("module1.print_hello".to_string(), Map::new())
			.await
			.unwrap();
		loop {
			task::sleep(std::time::Duration::from_millis(1000)).await;
		}
	};
	task::block_on(futures::future::join(module1, module2));
	*/
}
