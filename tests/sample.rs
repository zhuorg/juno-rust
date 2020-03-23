use async_std::task;
use gotham::GothamModule;
use std::collections::HashMap;

#[test]
fn sample() {
	task::block_on(async {
		let mut module = GothamModule::default(String::from("../gotham.sock"));
		module
			.initialize("module1".to_string(), "1.0.0".to_string(), HashMap::new())
			.await
			.await;
		println!("done");
		loop {
			task::sleep(std::time::Duration::from_millis(1000)).await;
		}
	});
}
