use async_std::{fs::remove_file, io::Result, os::unix::net::UnixListener, prelude::*, task};
use futures::future;
use futures_util::sink::SinkExt;
use juno::connection::{BaseConnection, UnixSocketConnection};

#[test]
fn connection_object_should_create_successfully() {
	let _socket_connection = UnixSocketConnection::new(String::from("socket_path"));
}

#[test]
fn should_connect() -> Result<()> {
	task::block_on(should_connect_async())
}

async fn should_connect_async() -> Result<()> {
	// Setup to try and connect to socket server
	let mut connection = UnixSocketConnection::new(String::from("./temp-1.sock"));

	// Listen for unix socket connections
	let socket = UnixListener::bind("./temp-1.sock").await?;
	let mut incoming = socket.incoming();
	let connection_listener = incoming.next();

	let (..) = future::join(connection_listener, connection.setup_connection()).await;

	remove_file("./temp-1.sock").await?;

	Ok(())
}

#[test]
fn should_connect_and_send_data() -> Result<()> {
	task::block_on(should_connect_and_send_data_async())
}

async fn should_connect_and_send_data_async() -> Result<()> {
	// Setup to try and connect to socket server
	let mut connection = UnixSocketConnection::new(String::from("./temp-2.sock"));

	// Listen for unix socket connections
	let socket = UnixListener::bind("./temp-2.sock").await?;
	let mut incoming = socket.incoming();
	let connection_listener = incoming.next();

	let (stream, _) = future::join(connection_listener, connection.setup_connection()).await;

	let mut read_buffer = [0; 10];
	let (_, read_result) = future::join(
		connection.send(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0]),
		stream.unwrap()?.read(&mut read_buffer),
	)
	.await;
	read_result?;

	assert_eq!(read_buffer.to_vec(), vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0]);

	remove_file("./temp-2.sock").await?;

	Ok(())
}

#[test]
fn should_connect_and_read_data() -> Result<()> {
	task::block_on(should_connect_and_read_data_async())
}

async fn should_connect_and_read_data_async() -> Result<()> {
	// Setup to try and connect to socket server
	let mut connection = UnixSocketConnection::new(String::from("./temp-3.sock"));

	// Listen for unix socket connections
	let socket = UnixListener::bind("./temp-3.sock").await?;
	let mut incoming = socket.incoming();
	let connection_listener = incoming.next();

	let (stream, _) = future::join(connection_listener, connection.setup_connection()).await;

	let write_data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0];
	let (write_action, read_buffer) = future::join(
		stream.unwrap()?.write_all(write_data.as_slice()),
		connection.get_data_receiver().next(),
	)
	.await;
	write_action?;
	let read_buffer = read_buffer.unwrap();

	assert_eq!(read_buffer, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0]);

	remove_file("./temp-3.sock").await?;

	Ok(())
}

#[test]
fn should_connect_and_send_data_from_cloned_sender() -> Result<()> {
	task::block_on(should_connect_and_send_data_from_cloned_sender_async())
}

async fn should_connect_and_send_data_from_cloned_sender_async() -> Result<()> {
	// Setup to try and connect to socket server
	let mut connection = UnixSocketConnection::new(String::from("./temp-4.sock"));

	// Listen for unix socket connections
	let socket = UnixListener::bind("./temp-4.sock").await?;
	let mut incoming = socket.incoming();
	let connection_listener = incoming.next();

	let (stream, _) = future::join(connection_listener, connection.setup_connection()).await;

	let mut read_buffer = [0; 10];
	let (write_action, read_action) = future::join(
		connection
			.clone_write_sender()
			.send(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0]),
		stream.unwrap()?.read(&mut read_buffer),
	)
	.await;
	write_action.unwrap();
	read_action?;

	assert_eq!(read_buffer.to_vec(), vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0]);

	remove_file("./temp-4.sock").await?;

	Ok(())
}

#[test]
#[should_panic]
fn should_send_data_without_connection_and_panic() {
	let mut connection = UnixSocketConnection::new(String::from("./test.sock"));
	task::block_on(connection.send(vec![]));
}

#[test]
#[should_panic]
fn should_close_connection_without_setup_and_panic() {
	let mut connection = UnixSocketConnection::new(String::from("./test.sock"));
	task::block_on(connection.close_connection());
}

#[test]
#[should_panic]
fn should_get_data_receiver_without_setup_and_panic() {
	let mut connection = UnixSocketConnection::new(String::from("./test.sock"));
	connection.get_data_receiver();
}

#[test]
#[should_panic]
fn should_clone_write_sender_without_setup_and_panic() {
	let connection = UnixSocketConnection::new(String::from("./test.sock"));
	connection.clone_write_sender();
}

#[test]
#[should_panic]
fn should_setup_connection_twice_and_panic() {
	let handle = std::thread::spawn(|| {
		task::block_on(should_setup_connection_twice_and_panic_async()).unwrap();
	})
	.join();
	task::block_on(remove_file("./temp-5.sock")).unwrap();
	handle.unwrap();
}

async fn should_setup_connection_twice_and_panic_async() -> Result<()> {
	// Setup to try and connect to socket server
	let mut connection = UnixSocketConnection::new(String::from("./temp-5.sock"));

	// Listen for unix socket connections
	let socket = UnixListener::bind("./temp-5.sock").await?;
	let mut incoming = socket.incoming();
	let _ = incoming.next();

	connection.setup_connection().await.unwrap();
	connection.setup_connection().await.unwrap();

	Ok(())
}
