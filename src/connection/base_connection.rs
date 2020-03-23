use crate::connection::Buffer;
use async_trait::async_trait;
use futures::channel::mpsc::UnboundedReceiver;

#[async_trait]
pub trait BaseConnection {
	async fn setup_connection(&mut self);
	async fn close_connection(&mut self);
	async fn send(&mut self, buffer: Buffer);

	fn get_data_receiver(&mut self) -> UnboundedReceiver<Buffer>;
}
