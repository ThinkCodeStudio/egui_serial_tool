use std::sync::Arc;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    sync::{mpsc::{Receiver, Sender}, Mutex},
};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

pub struct Serial {
    runtime: tokio::runtime::Runtime,
    run_state: Arc<Mutex<bool>>,
}

impl Serial {
    async fn read_task(
        mut rx: ReadHalf<SerialStream>,
        rx_sender: Sender<Vec<u8>>,
    ) {
        let mut buf = [0u8; 1024];
        loop {
            match rx.read(&mut buf).await {
                Ok(n) => {
                    rx_sender.send(buf[0..n].to_vec()).await.unwrap();
                }
                Err(e) => {
                    eprintln!("Read error: {}", e);
                    break;
                }
            }
        }
    }

    async fn write_task(
        mut tx: WriteHalf<SerialStream>,
        mut tx_receiver: Receiver<Vec<u8>>,
    ) {
        loop {
            if let Some(mut data) = tx_receiver.recv().await {
                tx.write_all(data.as_mut_slice()).await.unwrap();
            }
        }
    }

    pub fn new() -> Self {
        Self { 
            runtime: tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap(),
            run_state: Arc::new(Mutex::new(true)) 
        }
    }

    pub fn create(
        &mut self,
        serial_builder: tokio_serial::SerialPortBuilder,
    ) -> (Sender<Vec<u8>>, Receiver<Vec<u8>>) {
        let serial = serial_builder.open_native_async().unwrap();
        let (mut serial_rx, mut serial_tx) = tokio::io::split(serial);
        
        let (rx_sender, rx_receiver) = tokio::sync::mpsc::channel::<Vec<u8>>(64);
        let (tx_sender, mut tx_receiver) = tokio::sync::mpsc::channel::<Vec<u8>>(64);

        return (tx_sender, rx_receiver);
    }
}
