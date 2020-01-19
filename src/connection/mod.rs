
mod base_connection;
mod socket_connection;

pub use base_connection::BaseConnection;
pub use socket_connection::UnixSocketConnection;

pub struct OnDataHandler {
	pub function: fn(Buffer),
}

pub struct Buffer{
	pub data: Vec<u8>
}
