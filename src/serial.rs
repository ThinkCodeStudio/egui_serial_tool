use std::sync::Arc;

use tokio::{io::{AsyncReadExt, ReadHalf}, sync::mpsc::{Receiver, Sender}};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

pub struct Serial {
    serial: Arc<tokio_serial::SerialStream>,
}

impl Serial {
    async fn read_task(&mut self, rx:ReadHalf<SerialStream>, ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let rx_send = rx.clone();
            let mut buf = [0u8; 1024];
            loop {
                match rx.read(&mut buf).await {
                    Ok(n) => {
                        rx_send.send(buf.to_vec()).await.unwrap();
                    }
                    Err(e) => {
                        eprintln!("Read error: {}", e);
                        break;
                    }
                }
            }
        })
    }

    pub fn create(
        &mut self,
        serial_builder: tokio_serial::SerialPortBuilder,
    ) -> (Sender<Vec<u8>>, Receiver<Vec<u8>>) {
        self.serial = Arc::new(serial_builder.open_native_async().unwrap());
        let (rx_sender, mut rx_receiver) = tokio::sync::mpsc::channel::<Vec<u8>>(64);
        let (tx_sender, mut tx_receiver) = tokio::sync::mpsc::channel::<Vec<u8>>(64);
    }
}
