mod base_connection;
mod unix_socket_connection;

pub use base_connection::BaseConnection;
pub use unix_socket_connection::UnixSocketConnection;

pub type Buffer = Vec<u8>;
