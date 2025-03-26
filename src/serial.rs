use std::{sync::Arc, thread::sleep, time::Duration};

use tokio::{
    io::{AsyncReadExt, WriteHalf},
    sync::mpsc::{Receiver, Sender},
};
use tokio_serial::{SerialPortBuilder, SerialPortBuilderExt, SerialStream};

pub struct Serial {
    shutdown_tx: Option<Arc<Sender<()>>>,
    connection: bool,
    serial_writer: Option<WriteHalf<SerialStream>>,
}

impl Serial {
    pub fn new() -> Self {
        Serial {
            shutdown_tx: None,
            connection: false,
            serial_writer: None,
        }
    }

    pub fn connection(&mut self, serial_builder: SerialPortBuilder) {
        match serial_builder.open_native_async() {
            Ok(stream) => {
                let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(1);
                self.shutdown_tx = Some(Arc::new(tx));
                let (mut serial_rx, serial_tx) = tokio::io::split(stream);
                self.serial_writer = Some(serial_tx);
                self.connection = true;

                println!("startup");
                tokio::spawn(async move {
                    let mut buf = [0u8; 1024];
                    loop {
                        println!("loop");
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
                                println!("shutdowned");
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

    pub fn is_connected(&self) -> bool {
        self.connection
    }

    pub fn disconnection(&mut self) {
        if let Some(tx) = &self.shutdown_tx {
            let tx = Arc::clone(tx);
            tokio::spawn(async move {
                println!("shutdown1");
                tx.send(()).await.unwrap();
                println!("shutdown2");
            });
        }
        self.shutdown_tx = None;
        self.serial_writer = None;
        self.connection = false;
    }
}