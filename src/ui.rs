
use eframe::egui;
use tokio_serial::{available_ports, SerialPortInfo};

pub struct MainUi {
    serial_list: Vec<SerialPortInfo>,
    selected_port: SerialPortInfo,
}

impl Default for MainUi {
    fn default() -> Self {
        Self {
            serial_list: available_ports().unwrap_or(vec![]),
            selected_port: SerialPortInfo{
                port_name: String::from("Select Port"),
                port_type: tokio_serial::SerialPortType::Unknown,
            },
        }
    }
}

impl eframe::App for MainUi {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::ComboBox::from_label("Port")
                    .selected_text(format!("{}", self.selected_port.port_name))
                    .show_ui(ui, |ui| {
                        for port in &self.serial_list {
                            ui.selectable_value::<SerialPortInfo>(&mut self.selected_port, port.clone(), port.port_name.clone());
                        }
                    });
            });
        });
    }
}
