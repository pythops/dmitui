use std::fmt::Display;

use ratatui::{
    Frame,
    layout::{Constraint, Margin, Rect},
    style::Stylize,
    widgets::{Block, Cell, Padding, Row, Table},
};
use uuid::Uuid;

#[derive(Debug)]
pub struct System {
    manufacturer: String,
    product_name: String,
    version: String,
    serial_number: String,
    uuid: String,
    wakeup_type: WakeupType,
    sku: String,
    familly: String,
}

#[derive(Debug)]
enum WakeupType {
    Reserved,
    Unknown,
    ApmTimer,
    ModemRing,
    LanRemote,
    PowerSwitch,
    PciPme,
    AcPowerRestored,
    Other,
}

impl Display for WakeupType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reserved => write!(f, "Reserved"),
            Self::Unknown => write!(f, "Unknown"),
            Self::ApmTimer => write!(f, "APM Timer"),
            Self::ModemRing => write!(f, "Modem Ring"),
            Self::LanRemote => write!(f, "LAN Remote"),
            Self::PowerSwitch => write!(f, "Power Switch"),
            Self::PciPme => write!(f, "PCI PME"),
            Self::AcPowerRestored => write!(f, "AC Power Restored"),
            Self::Other => write!(f, "Other"),
        }
    }
}

impl From<u8> for WakeupType {
    fn from(value: u8) -> Self {
        match value {
            0 => WakeupType::Reserved,
            1 => WakeupType::Other,
            2 => WakeupType::Unknown,
            3 => WakeupType::ApmTimer,
            4 => WakeupType::ModemRing,
            5 => WakeupType::LanRemote,
            6 => WakeupType::PowerSwitch,
            7 => WakeupType::PciPme,
            8 => WakeupType::AcPowerRestored,
            _ => unreachable!(),
        }
    }
}

impl From<(Vec<u8>, Vec<String>)> for System {
    fn from((data, text): (Vec<u8>, Vec<String>)) -> Self {
        Self {
            manufacturer: text[data[0].saturating_sub(1) as usize].clone(),
            product_name: text[data[1].saturating_sub(1) as usize].clone(),
            version: text[data[2].saturating_sub(1) as usize].clone(),
            serial_number: text[data[3].saturating_sub(1) as usize].clone(),
            uuid: Uuid::from_bytes_le(data[4..20].try_into().unwrap()).to_string(),
            wakeup_type: WakeupType::from(data[20]),
            sku: text[data[21].saturating_sub(1) as usize].clone(),
            familly: text[data[22].saturating_sub(1) as usize].clone(),
        }
    }
}

impl System {
    pub fn render(&self, frame: &mut Frame, block: Rect) {
        let rows = vec![
            Row::new(vec![
                Cell::from("Manufacturer").bold(),
                Cell::from(self.manufacturer.clone()),
            ]),
            Row::new(vec![
                Cell::from("Product Name").bold(),
                Cell::from(self.product_name.clone()),
            ]),
            Row::new(vec![
                Cell::from("Version").bold(),
                Cell::from(self.version.clone()),
            ]),
            Row::new(vec![
                Cell::from("Serial Number").bold(),
                Cell::from(self.serial_number.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Uuid").bold(),
                Cell::from(self.uuid.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Wakeup Type").bold(),
                Cell::from(self.wakeup_type.to_string()),
            ]),
            Row::new(vec![
                Cell::from("SKU Number").bold(),
                Cell::from(self.sku.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Familly").bold(),
                Cell::from(self.familly.clone()),
            ]),
        ];
        let widths = [Constraint::Length(20), Constraint::Fill(1)];
        let table = Table::new(rows, widths).block(Block::new().padding(Padding::uniform(2)));

        frame.render_widget(table, block.inner(Margin::new(2, 0)));
    }
}
