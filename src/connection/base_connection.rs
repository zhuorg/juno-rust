use crate::connection::Buffer;
use async_trait::async_trait;
use futures::channel::mpsc::UnboundedSender;

#[async_trait]
pub trait BaseConnection {
	fn connect_and_listen(&mut self, socket_path: String, data_sender: UnboundedSender<Vec<u8>>);
	async fn close_connection(&mut self);
	async fn send(&mut self, buffer: Buffer);
}
