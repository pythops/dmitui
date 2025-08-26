use ratatui::{
    Frame,
    layout::{Constraint, Margin, Rect},
    style::Stylize,
    widgets::{Block, Cell, Padding, Row, Table},
};

#[derive(Debug)]
pub struct Battery {
    location: String,
    manufacturer: String,
    manufacture_date: String,
    serial_number: String,
    device_name: String,
    device_chemistry: String,
    design_capacity: Option<u16>,
    design_voltage: Option<u16>,
    sbds_version: String,
    max_error_in_battery: Option<u8>,
    oem_specific: u32,
}

impl From<(Vec<u8>, Vec<String>)> for Battery {
    fn from((data, text): (Vec<u8>, Vec<String>)) -> Self {
        let manufacture_date = if data[2] == 0 {
            let value = u16::from_le_bytes(data[14..16].try_into().unwrap());
            let day = (value & 0b11111) as u8;
            let month = ((value >> 5) & 0b1111) as u8;
            let year = (value >> 9) + 1980;

            format!("{}-{:02}-{:02}", year, month, day)
        } else {
            text[data[2].saturating_sub(1) as usize].clone()
        };

        let serial_number = if data[3] == 0 {
            format!(
                "0x{:X}",
                u16::from_le_bytes(data[12..14].try_into().unwrap())
            )
        } else {
            text[data[3].saturating_sub(1) as usize].clone()
        };

        let design_voltage = {
            let value = u16::from_le_bytes(data[8..10].try_into().unwrap());
            if value == 0 { None } else { Some(value) }
        };

        let device_chemistry = {
            if data[5] == 2 {
                text[data[16].saturating_sub(1) as usize].clone()
            } else {
                Chemistry::from(data[5]).to_string()
            }
        };

        let design_capacity = {
            let value = u16::from_le_bytes(data[6..8].try_into().unwrap());
            if value == 0 {
                None
            } else {
                Some(value * (data[17] as u16))
            }
        };

        let max_error_in_battery = {
            if data[11] == 0xFF {
                None
            } else {
                Some(data[11])
            }
        };

        Self {
            location: text[data[0].saturating_sub(1) as usize].clone(),
            manufacturer: text[data[1].saturating_sub(1) as usize].clone(),
            manufacture_date,
            serial_number,
            device_name: text[data[4].saturating_sub(1) as usize].clone(),
            device_chemistry,
            design_capacity,
            design_voltage,
            sbds_version: text[data[10].saturating_sub(1) as usize].clone(),
            max_error_in_battery,
            oem_specific: u32::from_le_bytes(data[18..22].try_into().unwrap()),
        }
    }
}

impl Battery {
    pub fn render(&self, frame: &mut Frame, block: Rect) {
        let rows = vec![
            Row::new(vec![
                Cell::from("location").bold(),
                Cell::from(self.location.clone()),
            ]),
            Row::new(vec![
                Cell::from("Manufacturer").bold(),
                Cell::from(self.manufacturer.clone()),
            ]),
            Row::new(vec![
                Cell::from("Manufacture Date").bold(),
                Cell::from(self.manufacture_date.clone()),
            ]),
            Row::new(vec![
                Cell::from("Serial number").bold(),
                Cell::from(self.serial_number.clone()),
            ]),
            Row::new(vec![
                Cell::from("Name").bold(),
                Cell::from(self.device_name.clone()),
            ]),
            Row::new(vec![
                Cell::from("Chemistry").bold(),
                Cell::from(self.device_chemistry.clone()),
            ]),
            Row::new(vec![
                Cell::from("Design Voltage").bold(),
                Cell::from({
                    if let Some(v) = self.design_voltage {
                        format!("{v} mV")
                    } else {
                        "Unknown".to_string()
                    }
                }),
            ]),
            Row::new(vec![
                Cell::from("Design Capacity").bold(),
                Cell::from({
                    if let Some(v) = self.design_capacity {
                        format!("{v} mWh")
                    } else {
                        "Unknown".to_string()
                    }
                }),
            ]),
            Row::new(vec![
                Cell::from("SBDS Version").bold(),
                Cell::from(self.sbds_version.clone()),
            ]),
            Row::new(vec![
                Cell::from("Maximum Error").bold(),
                Cell::from({
                    if let Some(v) = self.max_error_in_battery {
                        format!("{v}%")
                    } else {
                        "Unknown".to_string()
                    }
                }),
            ]),
            Row::new(vec![
                Cell::from("OEM specific").bold(),
                Cell::from(format!("0x{:X}", self.oem_specific)),
            ]),
        ];

        let widths = [Constraint::Length(20), Constraint::Fill(1)];
        let table = Table::new(rows, widths).block(Block::new().padding(Padding::uniform(2)));
        frame.render_widget(table, block.inner(Margin::new(2, 0)));
    }
}

#[derive(Debug, strum::Display)]
enum Chemistry {
    #[strum(to_string = "Other")]
    Other,
    #[strum(to_string = "Unknown")]
    Unknown,
    #[strum(to_string = "Lead Acid")]
    LeadAcid,
    #[strum(to_string = "Nickel Cadmium")]
    NickelCadmium,
    #[strum(to_string = "Nickel metal hydride")]
    NickelMetalHydride,
    #[strum(to_string = "Lithium-ion")]
    LithiumIon,
    #[strum(to_string = "Zinc air")]
    ZincAir,
    #[strum(to_string = "Lithium Polymer")]
    LithiumPolymer,
}

impl From<u8> for Chemistry {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Other,
            2 => Self::Unknown,
            3 => Self::LeadAcid,
            4 => Self::NickelCadmium,
            5 => Self::NickelMetalHydride,
            6 => Self::LithiumIon,
            7 => Self::ZincAir,
            8 => Self::LithiumPolymer,
            _ => unreachable!(),
        }
    }
}
