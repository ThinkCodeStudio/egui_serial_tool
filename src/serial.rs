use std::sync::Arc;

use tokio::{
    io::{AsyncReadExt, WriteHalf},
    sync::mpsc::{Receiver, Sender},
};
use tokio_serial::{SerialPortBuilder, SerialStream};

struct Serial {
    shutdown_tx: Option<Arc<Sender<()>>>,
    connection: bool,
    serial_writer: Option<WriteHalf<SerialStream>>,
}

impl Serial {
    fn new() -> Self {
        Serial {
            shutdown_tx: None,
            connection: false,
            serial_writer: None,
        }
    }

    fn connection(&mut self, serial_builder: &SerialPortBuilder) {
        match SerialStream::open(serial_builder) {
            Ok(stream) => {
                let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(1);
                self.shutdown_tx = Some(Arc::new(tx));
                let (mut serial_rx, serial_tx) = tokio::io::split(stream);
                self.serial_writer = Some(serial_tx);
                self.connection = true;

                tokio::spawn(async move {
                    let mut buf = [0u8; 1024];
                    loop {
                        tokio::select! {
                            ret = serial_rx.read(&mut buf) => {
                                match ret {
                                    Ok(n) => {
                                        println!("Read size: {}", n);
                                    }
                                    Err(e) => {
                                        eprintln!("Read error: {}", e);
                                        break;
                                    }
                                }
                            }

                            _ = rx.recv() => {
                                break;
                            }
                        }
                    }
                    println!("task end");
                });
            }
            Err(e) => {
                eprintln!("Read error: {}", e);
            }
        }
    }

    fn disconnection(&mut self) {
        if let Some(tx) = &self.shutdown_tx {
            let tx = Arc::clone(tx);
            tokio::spawn(async move {
                tx.send(()).await.unwrap();
            });
        }
        self.shutdown_tx = None;
        self.serial_writer = None;
        self.connection = false;
    }
}

#[tokio::test]
async fn test_serial() {
    let mut serial = Serial::new();
    let serial_builder = tokio_serial::new("COM4", 115200)
        .data_bits(tokio_serial::DataBits::Eight)
        .stop_bits(tokio_serial::StopBits::One)
        .parity(tokio_serial::Parity::None);
    
    serial.connection(&serial_builder);
    assert_eq!(serial.connection, true);
    serial.disconnection();
    assert_eq!(serial.connection, false);
    // no shutdown
    serial.connection(&serial_builder);
    assert_eq!(serial.connection, true);
    serial.disconnection();
    assert_eq!(serial.connection, false);
}