use crate::connection::{Buffer, OnDataHandler};

pub trait BaseConnection {
	fn setup_connection(&mut self);
	fn close_connection(&mut self);
	fn send(&mut self, buffer: Buffer);

	fn set_on_data_listener(&mut self, on_data_handler: OnDataHandler);

	fn on_data(&self, data: Buffer);
}
