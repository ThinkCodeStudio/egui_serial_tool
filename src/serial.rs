use std::{fmt::Error, sync::Arc};
use tokio::sync::Mutex;
use tokio_serial::SerialPortBuilder;

pub struct Serial {
    connection_mutex: Arc<Mutex<bool>>,
    serial_writer: Option<tokio::sync::mpsc::Sender<Vec<u8>>>,
    serial_reader: Option<tokio::sync::mpsc::Receiver<Vec<u8>>>,
    connection: bool,
}

impl Serial {
    pub fn new() -> Self {
        Serial {
            connection_mutex: Arc::new(Mutex::new(false)),
            serial_writer: None,
            serial_reader: None,
            connection: false,
        }
    }

    pub fn connection(&mut self, serial_builder: SerialPortBuilder) -> Result<(), Error> {
        if self.connection {
            return Err(Error);
        }
        self.connection = true;
        let connection_mutex = Arc::clone(&self.connection_mutex);
        let (write_tx, mut write_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(64);
        self.serial_writer = Some(write_tx);
        let (read_tx, read_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(64);
        self.serial_reader = Some(read_rx);
        tokio::spawn(async move {
            match serial_builder.open() {
                Ok(mut port) => {
                    {
                        let mut connection = connection_mutex.lock().await;
                        *connection = true;
                    }
                    let mut buf = [0u8; 1024];
                    loop {
                        let connection = connection_mutex.lock().await;
                        if *connection == false {
                            break;
                        }
                        match port.read(&mut buf) {
                            Ok(n) => {
                                if n == 0 {
                                    continue;
                                }
                                let data = &buf[..n];
                                read_tx.send(data.to_vec()).await.unwrap();
                            }
                            Err(e) => {
                                eprintln!("Read error: {}", e);
                            }
                        }

                        match write_rx.recv().await {
                            Some(data) => {
                                if let Err(e) = port.write_all(&data) {
                                    eprintln!("Write error: {}", e);
                                    break;
                                }
                            }
                            None => {
                                println!("write_rx closed");
                            }
                        }
                    }
                    println!("serial task closed");
                }
                Err(e) => {
                    eprintln!("Serial open error: {}", e);
                }
            }
        });

        return Ok(());
    }

    pub fn is_connected(&self) -> bool {
        self.connection
    }

    pub async fn send(&self, data: Vec<u8>) -> Result<(), Error> {
        if let Some(writer) = &self.serial_writer {
            writer.send(data).await.unwrap();
            Ok(())
        } else {
            Err(Error)
        }
    }

    pub async fn receive(&mut self) -> Result<Option<Vec<u8>>, Error> {
        if let Some(reader) = &mut self.serial_reader {
            match reader.recv().await {
                Some(data) => Ok(Some(data)),
                None => Ok(None),
            }
        } else {
            Err(Error)
        }
    }

    pub fn disconnection(&mut self) {
        let connection_mutex = Arc::clone(&self.connection_mutex);
        tokio::spawn(async move {
            let mut connection = connection_mutex.lock().await;
            *connection = false;
        });
        self.serial_writer = None;
        self.serial_reader = None;
        self.connection = false;
    }
}
