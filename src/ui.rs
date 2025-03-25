use std::sync::{Arc, Mutex};
use std::thread;

use crate::loader::load_baud;
use eframe::egui;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::{
    available_ports, DataBits, Parity, SerialPortBuilderExt, SerialPortInfo, StopBits,
};

const BAUD_FILE_PATH: &str = "baud.ini";

#[derive(PartialEq, Eq)]
enum SerialState {
    DISCONNECTED,
    CONNECTED,
}

pub struct MainUi {
    serial_list: Vec<SerialPortInfo>,
    baud_list: Vec<u32>,

    selected_port: SerialPortInfo,
    selected_baud: u32,
    selected_data_bits: DataBits,
    selected_stop_bits: StopBits,
    selected_parity: Parity,
    en_connect: bool,
    serial_state: Arc<Mutex<SerialState>>,
}

impl Default for MainUi {
    fn default() -> Self {
        Self {
            serial_list: available_ports().unwrap_or(vec![]),
            baud_list: load_baud(BAUD_FILE_PATH),

            selected_port: SerialPortInfo {
                port_name: String::from("Select Port"),
                port_type: tokio_serial::SerialPortType::Unknown,
            },
            selected_baud: 115200,
            selected_data_bits: DataBits::Eight,
            selected_stop_bits: StopBits::One,
            selected_parity: Parity::None,
            en_connect: true,
            serial_state: Arc::new(Mutex::new(SerialState::DISCONNECTED)),
        }
    }
}

impl MainUi {
    fn connection(&mut self) {
        let (rx_sender, mut rx_receiver) = tokio::sync::mpsc::channel::<Vec<u8>>(64);
        let (tx_sender, mut tx_receiver) = tokio::sync::mpsc::channel::<Vec<u8>>(64);

        if let Ok(mut state) = self.serial_state.lock(){
            *state = SerialState::CONNECTED;
        }

        let serial_builder =
            tokio_serial::new(self.selected_port.port_name.clone(), self.selected_baud)
                .data_bits(self.selected_data_bits)
                .stop_bits(self.selected_stop_bits)
                .parity(self.selected_parity);

        let serial_mutex = Arc::clone(&self.serial_state);
        thread::spawn(async move || {
            if let Ok(serial) = serial_builder.open_native_async() {
                let (mut serial_rx, mut serial_tx) = tokio::io::split(serial);
                
                let resd_task = tokio::spawn(async move {
                    let rx_send = rx_sender.clone();
                    let mut buf = [0u8; 1024];
                    loop {
                        match serial_rx.read(&mut buf).await {
                            Ok(n) => {
                                rx_send.send(buf.to_vec()).await.unwrap();
                            }
                            Err(e) => {
                                eprintln!("Read error: {}", e);
                                break;
                            }
                        }
                    }
                });

                let write_task = tokio::spawn(async move {
                    loop{
                        if let Some(mut data) = tx_receiver.recv().await {
                            serial_tx.write_all(data.as_mut_slice()).await.unwrap();
                        }
                    }
                    
                });

                let _ = tokio::join!(resd_task, write_task);
            }
        });
    }

    fn disconnection(&mut self) {
        if let Ok(mut state) = self.serial_state.lock(){
            *state = SerialState::DISCONNECTED;
        }
    }
}

impl eframe::App for MainUi {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(self.en_connect, egui::Button::new("🔄"))
                    .clicked()
                {
                    self.serial_list = available_ports().unwrap_or(vec![])
                }

                ui.add_enabled_ui(self.en_connect, |ui| {
                    egui::ComboBox::from_label("Port")
                        .selected_text(format!("{}", self.selected_port.port_name))
                        .show_ui(ui, |ui| {
                            for port in &self.serial_list {
                                ui.selectable_value::<SerialPortInfo>(
                                    &mut self.selected_port,
                                    port.clone(),
                                    port.port_name.clone(),
                                );
                            }
                        });
                });

                ui.add_enabled_ui(self.en_connect, |ui| {
                    egui::ComboBox::from_label("Baud")
                        .selected_text(format!("{}", self.selected_baud))
                        .show_ui(ui, |ui| {
                            for baud in &self.baud_list {
                                ui.selectable_value(
                                    &mut self.selected_baud,
                                    baud.clone(),
                                    baud.to_string(),
                                );
                            }
                        });
                });

                ui.add_enabled_ui(self.en_connect, |ui| {
                    egui::ComboBox::from_label("Data Bits")
                        .selected_text(format!("{}", u8::from(self.selected_data_bits)))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.selected_data_bits,
                                DataBits::Five,
                                format!("{}", u8::from(DataBits::Five)),
                            );
                            ui.selectable_value(
                                &mut self.selected_data_bits,
                                DataBits::Six,
                                format!("{}", u8::from(DataBits::Six)),
                            );
                            ui.selectable_value(
                                &mut self.selected_data_bits,
                                DataBits::Seven,
                                format!("{}", u8::from(DataBits::Seven)),
                            );
                            ui.selectable_value(
                                &mut self.selected_data_bits,
                                DataBits::Eight,
                                format!("{}", u8::from(DataBits::Eight)),
                            );
                        });
                });

                ui.add_enabled_ui(self.en_connect, |ui| {
                    egui::ComboBox::from_label("Stop Bits")
                        .selected_text(format!("{}", u8::from(self.selected_stop_bits)))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.selected_stop_bits,
                                StopBits::One,
                                format!("{}", u8::from(StopBits::One)),
                            );

                            ui.selectable_value(
                                &mut self.selected_stop_bits,
                                StopBits::Two,
                                format!("{}", u8::from(StopBits::Two)),
                            );
                        });
                });

                ui.add_enabled_ui(self.en_connect, |ui| {
                    egui::ComboBox::from_label("Parity")
                        .selected_text(format!("{}", self.selected_parity))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.selected_parity,
                                Parity::None,
                                format!("{}", Parity::None),
                            );
                            ui.selectable_value(
                                &mut self.selected_parity,
                                Parity::Even,
                                format!("{}", Parity::Even),
                            );
                            ui.selectable_value(
                                &mut self.selected_parity,
                                Parity::Odd,
                                format!("{}", Parity::Odd),
                            );
                        });
                });

                if ui
                    .add(egui::Button::new(if self.en_connect {
                        "connection"
                    } else {
                        "disconnection"
                    }))
                    .clicked()
                {
                    if self.selected_port.port_type != tokio_serial::SerialPortType::Unknown {
                        if self.en_connect {
                            self.connection();
                        } else {
                            self.disconnection();
                        }
                        self.en_connect = !self.en_connect;
                    }
                }
            });
        });
    }
}
