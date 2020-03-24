use async_std::{fs::remove_file, io::Result, os::unix::net::UnixListener, prelude::*, task};
use futures_util::sink::SinkExt;
use gotham::connection::{BaseConnection, UnixSocketConnection};

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

	let (..) = futures::future::join(connection_listener, connection.setup_connection()).await;

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

	let (stream, _) =
		futures::future::join(connection_listener, connection.setup_connection()).await;

	connection.send(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0]).await;

	let mut read_buffer = [0; 10];
	stream.unwrap()?.read(&mut read_buffer).await?;

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

	let (stream, _) =
		futures::future::join(connection_listener, connection.setup_connection()).await;

	let write_data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0];
	stream.unwrap()?.write_all(write_data.as_slice()).await?;

	let read_buffer = connection.get_data_receiver().next().await.unwrap();

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

	let (stream, _) =
		futures::future::join(connection_listener, connection.setup_connection()).await;

	connection
		.clone_write_sender()
		.send(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0])
		.await
		.unwrap();

	let mut read_buffer = [0; 10];
	stream.unwrap()?.read(&mut read_buffer).await?;

	assert_eq!(read_buffer.to_vec(), vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0]);

	remove_file("./temp-4.sock").await?;

	Ok(())
}
