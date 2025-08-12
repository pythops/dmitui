#![allow(warnings)]

use ratatui::{
    Frame,
    layout::{Constraint, Margin, Rect},
    style::Stylize,
    widgets::{Block, Cell, Padding, Row, Table},
};

#[derive(Debug)]
pub struct Chassis {
    manufacturer: String,
    chassis_type: ChassisType,
    lock: bool,
    version: String,
    serial_number: String,
    asset_tag_number: String,
    bootup_state: State,
    power_supply_state: State,
    thermal_state: State,
    security_status: SecurityStatus,
    oem_defined: u32,
    height: Option<u8>,
    number_power_cords: Option<u8>,
    contained_element_count: u8,
    contained_element_record_length: u8,
    contained_elements: Vec<u32>,
    sku_number: String,
}

#[derive(Debug, strum::Display)]
enum ChassisType {
    #[strum(to_string = "Other")]
    Other,
    #[strum(to_string = "Unknown")]
    Unknown,
    #[strum(to_string = "Desktop")]
    Desktop,
    #[strum(to_string = "Low Profile Desktop")]
    LowProfileDesktop,
    #[strum(to_string = "Pizza Box ")]
    PizzaBox,
    #[strum(to_string = "Mini Tower")]
    MiniTower,
    #[strum(to_string = "Tower")]
    Tower,
    #[strum(to_string = "Portable")]
    Portable,
    #[strum(to_string = "Laptop")]
    Laptop,
    #[strum(to_string = "Notebook")]
    Notebook,
    #[strum(to_string = "Hand Held")]
    HandHeld,
    #[strum(to_string = "Docking Station")]
    DockingStation,
    #[strum(to_string = "All in One")]
    AllInOne,
    #[strum(to_string = "Sub Notebook")]
    SubNotebook,
    #[strum(to_string = "Space-saving")]
    SpaceSaving,
    #[strum(to_string = "Lunch Box")]
    LunchBox,
    #[strum(to_string = "Main Server Chassis")]
    MainServerChassis,
    #[strum(to_string = "Expansion Chassis")]
    ExpansionChassis,
    #[strum(to_string = "SubChassis")]
    SubChassis,
    #[strum(to_string = "Bus Expansion Chassis")]
    BusExpansionChassis,
    #[strum(to_string = "Peripheral Chassis")]
    PeripheralChassis,
    #[strum(to_string = "RAID Chassis")]
    RAIDChassis,
    #[strum(to_string = "Rack Mount Chassis")]
    RackMountChassis,
    #[strum(to_string = "Sealed-case PC")]
    SealedcasePC,
    #[strum(to_string = "Multi-system chassis")]
    MultisystemChassis,
    #[strum(to_string = "Compact PCI")]
    CompactPCI,
    #[strum(to_string = "Advanced TCA")]
    AdvancedTCA,
    #[strum(to_string = "Blade")]
    Blade,
    #[strum(to_string = "Blade Enclosure")]
    BladeEncolsure,
    #[strum(to_string = "Tablet")]
    Tablet,
    #[strum(to_string = "Convertible")]
    Convertible,
    #[strum(to_string = "Detachable")]
    Detachable,
    #[strum(to_string = "IoT Gateway")]
    IoTGateway,
    #[strum(to_string = "Embedded PC")]
    EmbeddedPC,
    #[strum(to_string = "Mini PC")]
    MiniPC,
    #[strum(to_string = "Stick PC")]
    StickPC,
}

impl From<u8> for ChassisType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Other,
            2 => Self::Unknown,
            3 => Self::Desktop,
            4 => Self::LowProfileDesktop,
            5 => Self::PizzaBox,
            6 => Self::MiniTower,
            7 => Self::Tower,
            8 => Self::Portable,
            9 => Self::Laptop,
            10 => Self::Notebook,
            11 => Self::HandHeld,
            12 => Self::DockingStation,
            13 => Self::AllInOne,
            14 => Self::SubNotebook,
            15 => Self::SpaceSaving,
            16 => Self::LunchBox,
            17 => Self::MainServerChassis,
            18 => Self::ExpansionChassis,
            19 => Self::SubChassis,
            20 => Self::BusExpansionChassis,
            21 => Self::PeripheralChassis,
            22 => Self::RAIDChassis,
            23 => Self::RackMountChassis,
            24 => Self::SealedcasePC,
            25 => Self::MultisystemChassis,
            26 => Self::CompactPCI,
            27 => Self::AdvancedTCA,
            28 => Self::Blade,
            29 => Self::BladeEncolsure,
            30 => Self::Tablet,
            31 => Self::Convertible,
            32 => Self::Detachable,
            33 => Self::IoTGateway,
            34 => Self::EmbeddedPC,
            35 => Self::MiniPC,
            36 => Self::StickPC,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, strum::Display)]
enum State {
    #[strum(to_string = "Other")]
    Other,
    #[strum(to_string = "Unknown")]
    Unknown,
    #[strum(to_string = "Safe")]
    Safe,
    #[strum(to_string = "Warning")]
    Warning,
    #[strum(to_string = "Critical")]
    Critical,
    #[strum(to_string = "Non Recoverable")]
    NonRecoverable,
}

impl From<u8> for State {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Other,
            2 => Self::Unknown,
            3 => Self::Safe,
            4 => Self::Warning,
            5 => Self::Critical,
            6 => Self::NonRecoverable,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, strum::Display)]
enum SecurityStatus {
    #[strum(to_string = "Other")]
    Other,
    #[strum(to_string = "Unknown")]
    Unknown,
    #[strum(to_string = "None")]
    None,
    #[strum(to_string = "External interface locked out")]
    ExternalInterfaceLockedout,
    #[strum(to_string = "External interface enabled")]
    ExternalInterfaceEnabled,
}

impl From<u8> for SecurityStatus {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Other,
            2 => Self::Unknown,
            3 => Self::None,
            4 => Self::ExternalInterfaceLockedout,
            5 => Self::ExternalInterfaceEnabled,
            _ => unreachable!(),
        }
    }
}

impl From<(Vec<u8>, Vec<String>)> for Chassis {
    fn from((data, text): (Vec<u8>, Vec<String>)) -> Self {
        Self {
            manufacturer: text[data[0].saturating_sub(1) as usize].clone(),
            chassis_type: ChassisType::from(data[1]),
            lock: data[1] & (1 << 7) != 0,
            version: text[data[2].saturating_sub(1) as usize].clone(),
            serial_number: text[data[3].saturating_sub(1) as usize].clone(),
            asset_tag_number: text[data[4].saturating_sub(1) as usize].clone(),
            bootup_state: State::from(data[5]),
            power_supply_state: State::from(data[6]),
            thermal_state: State::from(data[7]),
            security_status: SecurityStatus::from(data[8]),
            oem_defined: u32::from_le_bytes(data[9..13].try_into().unwrap()),
            height: if data[13] == 0 { None } else { Some(data[13]) },
            number_power_cords: if data[14] == 0 { None } else { Some(data[14]) },
            contained_element_count: data[15],
            contained_element_record_length: data[16],
            contained_elements: Vec::new(),
            sku_number: text[data[17 + (data[15] * data[16]) as usize].saturating_sub(1) as usize]
                .clone(),
        }
    }
}

impl Chassis {
    pub fn render(&self, frame: &mut Frame, block: Rect) {
        let rows = vec![
            Row::new(vec![
                Cell::from("Manufacturer").bold(),
                Cell::from(self.manufacturer.clone()),
            ]),
            Row::new(vec![
                Cell::from("Type").bold(),
                Cell::from(self.chassis_type.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Lock").bold(),
                Cell::from(if self.lock {
                    "Present".to_string()
                } else {
                    "Not present".to_string()
                }),
            ]),
            Row::new(vec![
                Cell::from("Version").bold(),
                Cell::from(self.version.clone()),
            ]),
            Row::new(vec![
                Cell::from("Serial number").bold(),
                Cell::from(self.serial_number.clone()),
            ]),
            Row::new(vec![
                Cell::from("Asset Tag Number").bold(),
                Cell::from(self.asset_tag_number.clone()),
            ]),
            Row::new(vec![
                Cell::from("Bootup State").bold(),
                Cell::from(self.bootup_state.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Power Supply State").bold(),
                Cell::from(self.power_supply_state.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Thermal State").bold(),
                Cell::from(self.thermal_state.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Security Status").bold(),
                Cell::from(self.security_status.to_string()),
            ]),
            Row::new(vec![
                Cell::from("OEM defined").bold(),
                Cell::from(format!("0x{:X}", self.oem_defined)),
            ]),
            Row::new(vec![
                Cell::from("Height").bold(),
                Cell::from(if let Some(h) = self.height {
                    format!("{h}U")
                } else {
                    "Unspecified".to_string()
                }),
            ]),
            Row::new(vec![
                Cell::from("Number of Power Cords").bold(),
                Cell::from(if let Some(n) = self.number_power_cords {
                    n.to_string()
                } else {
                    "Unspecified".to_string()
                }),
            ]),
            Row::new(vec![
                Cell::from("contained elements").bold(),
                Cell::from(self.contained_element_count.to_string()),
            ]),
            Row::new(vec![
                Cell::from("SKU").bold(),
                Cell::from(self.sku_number.to_string()),
            ]),
        ];

        let widths = [Constraint::Length(20), Constraint::Fill(1)];
        let table = Table::new(rows, widths).block(Block::new().padding(Padding::uniform(2)));
        frame.render_widget(table, block.inner(Margin::new(2, 0)));
    }
}

//TODO: implement Contained Elements
