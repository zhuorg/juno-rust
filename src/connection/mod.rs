mod base_connection;
#[cfg(target_family = "unix")]
mod unix_socket_connection;
mod inet_socket_connection;

pub use base_connection::BaseConnection;
#[cfg(target_family = "unix")]
pub use unix_socket_connection::UnixSocketConnection;
pub use inet_socket_connection::InetSocketConnection;

pub type Buffer = Vec<u8>;
