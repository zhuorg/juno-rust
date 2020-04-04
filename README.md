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
    let mut module = GothamModule::default("./path/to/gotham.sock");
    // The hashmap below is used to mark dependencies
    module
        .initialize("module-name", "1.0.0", HashMap::new())
        .await
        .unwrap();
    println!("Initialized!");
    module
        .declare_function("print_hello", |args| {
            println!("Hello");
            Value::Null
        })
        .await
        .unwrap();
    // The HashMap::new() below marks the arguments passed to the function
    module
        .call_function("module2.print_hello_world", HashMap::new())
        .await
        .unwrap();
    loop {
        task::sleep(Duration::from_millis(1000)).await;
    }
}
```