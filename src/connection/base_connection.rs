use crate::connection::Buffer;
use async_trait::async_trait;

#[async_trait]
pub trait BaseConnection {
	async fn setup_connection(&mut self);
	async fn close_connection(&mut self);
	async fn send(&mut self, buffer: Buffer);
	async fn read_data(&self) -> Option<Buffer>;
}
