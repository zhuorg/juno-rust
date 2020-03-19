mod base_connection;
mod socket_connection;

pub use base_connection::BaseConnection;
pub use socket_connection::UnixSocketConnection;

pub type Buffer = Vec<u8>;
