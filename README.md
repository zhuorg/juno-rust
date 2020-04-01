# Gotham Rust

This is a library that provides you with helper methods for interfacing with the microservices framework, [gotham](https://github.com/bytesonus/gotham).

## How to use:

There is a lot of flexibility provided by the library, in terms of connection options and encoding protocol options. However, in order to use the library, none of that is required.

In case you are planning to implement a custom connection option, you will find an example in `src/connection/unix_socket_connection.rs`.

For all other basic needs, you can get away without worrying about any of that.

### A piece of code is worth a thousand words

```rust
use async_std::task;
use gotham::{models::Value, GothamModule};
use std::{time::Duration, collections::HashMap};

#[async_std::main]
async fn main() {
    let mut module = GothamModule::default(String::from("../path/to/gotham.sock"));
    module
        .initialize("module-name".to_string(), "1.0.0".to_string(), HashMap::new())
        .await
        .unwrap();
    // The hashmap is used to mark dependencies
    println!("Initialized!");
    module
        .declare_function("print_hello".to_string(), |args| {
            println!("Hello");
            Value::Null
        })
        .await
        .unwrap();
    module
        .call_function("module2.print_hello_world".to_string(), HashMap::new())
        .await
        .unwrap();
    // The HashMap::new() here marks the arguments passed to the function
    loop {
        task::sleep(Duration::from_millis(1000)).await;
    }
}
```