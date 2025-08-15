use ratatui::{
    Frame,
    layout::{Constraint, Margin, Rect},
    style::Stylize,
    widgets::{Block, Cell, Padding, Row, Table},
};

#[derive(Debug)]
pub struct Memory {
    pub physical_memory_array: PhysicalMemoryArray,
}

impl Memory {
    pub fn render(&mut self, frame: &mut Frame, block: Rect) {
        self.physical_memory_array.render(frame, block);
    }
}

#[derive(Debug)]
pub struct PhysicalMemoryArray {
    location: Location,
    function: Function,
    error_correction: ErrorCorrection,
    max_capacity: String,
    error_information_handle: Option<u16>,
    number_memory_devices: u16,
}

impl From<&[u8]> for PhysicalMemoryArray {
    fn from(data: &[u8]) -> Self {
        let max_capacity = {
            let value = u32::from_le_bytes(data[3..7].try_into().unwrap());

            if value == 0x80008000 {
                format!("{}T", u64::from_le_bytes(data[11..19].try_into().unwrap()))
            } else {
                match value {
                    value if value <= 1024 => {
                        format!("{value}K")
                    }
                    value if value <= 1024 * 1024 => {
                        format!("{}M", value / 1024)
                    }
                    _ => {
                        format!("{}G", value / 1024 / 1024)
                    }
                }
            }
        };
        let error_information_handle = {
            let value = u16::from_le_bytes(data[7..9].try_into().unwrap());

            if value == 0xFFFE { None } else { Some(value) }
        };

        let number_memory_devices = u16::from_le_bytes(data[9..11].try_into().unwrap());

        Self {
            location: Location::from(data[0]),
            function: Function::from(data[1]),
            error_correction: ErrorCorrection::from(data[2]),
            max_capacity,
            error_information_handle,
            number_memory_devices,
        }
    }
}

impl PhysicalMemoryArray {
    pub fn render(&self, frame: &mut Frame, block: Rect) {
        let rows = vec![
            Row::new(vec![
                Cell::from("Location").bold(),
                Cell::from(self.location.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Use").bold(),
                Cell::from(self.function.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Error Correction").bold(),
                Cell::from(self.error_correction.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Maximum Capacity").bold(),
                Cell::from(self.max_capacity.clone()),
            ]),
            Row::new(vec![
                Cell::from("Error Information Handle ").bold(),
                Cell::from({
                    if let Some(v) = self.error_information_handle {
                        v.to_string()
                    } else {
                        "Not Provided".to_string()
                    }
                }),
            ]),
            Row::new(vec![
                Cell::from("Number Of Devices").bold(),
                Cell::from(self.number_memory_devices.to_string()),
            ]),
        ];

        let widths = [Constraint::Length(30), Constraint::Fill(1)];
        let table = Table::new(rows, widths).block(Block::new().padding(Padding::uniform(2)));
        frame.render_widget(table, block.inner(Margin::new(2, 0)));
    }
}

#[derive(Debug, strum::Display)]
enum Location {
    #[strum(to_string = "Other")]
    Other,
    #[strum(to_string = "Unknown")]
    Unknown,
    #[strum(to_string = "System board or motherboard")]
    SystemBoard,
    #[strum(to_string = "ISA addon-on card")]
    Isa,
    #[strum(to_string = "EISA addon-on card")]
    Eisa,
    #[strum(to_string = "PCI addon-on card")]
    Pci,
    #[strum(to_string = "MCA addon-on card")]
    Mca,
    #[strum(to_string = "PCMCIA addon-on card")]
    Pcmcia,
    #[strum(to_string = "Proprietary addon-on card")]
    Proprietary,
    #[strum(to_string = "NuBus")]
    NuBus,
    #[strum(to_string = "PC-98/C20 add-on card")]
    Pc98C20,
    #[strum(to_string = "PC-98/C24 add-on card")]
    Pc98C24,
    #[strum(to_string = "PC-98/E add-on card")]
    Pc98E,
    #[strum(to_string = "PC-98/Local bus add-on card")]
    Pc98Local,
    #[strum(to_string = "CXL add-on card")]
    Cxl,
}

impl From<u8> for Location {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Other,
            2 => Self::Unknown,
            3 => Self::SystemBoard,
            4 => Self::Isa,
            5 => Self::Eisa,
            6 => Self::Pci,
            7 => Self::Mca,
            8 => Self::Pcmcia,
            9 => Self::Proprietary,
            10 => Self::NuBus,
            11 => Self::Pc98C20,
            12 => Self::Pc98C24,
            13 => Self::Pc98E,
            14 => Self::Pc98Local,
            15 => Self::Cxl,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, strum::Display)]
enum Function {
    #[strum(to_string = "Other")]
    Other,
    #[strum(to_string = "Unknown")]
    Unknown,
    #[strum(to_string = "System Memory")]
    SystemMemory,
    #[strum(to_string = "Video Memory")]
    VideoMemory,
    #[strum(to_string = "Flash Memory")]
    FlashMemory,
    #[strum(to_string = "Non-volatile RAM")]
    NonVolatileRAM,
    #[strum(to_string = "Cache Memory")]
    CacheMemory,
}

impl From<u8> for Function {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Other,
            2 => Self::Unknown,
            3 => Self::SystemMemory,
            4 => Self::VideoMemory,
            5 => Self::FlashMemory,
            6 => Self::NonVolatileRAM,
            7 => Self::CacheMemory,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, strum::Display)]
enum ErrorCorrection {
    #[strum(to_string = "Other")]
    Other,
    #[strum(to_string = "Unknown")]
    Unknown,
    #[strum(to_string = "None")]
    None,
    #[strum(to_string = "Parity")]
    Parity,
    #[strum(to_string = "Single-bit ECC")]
    SingleBitECC,
    #[strum(to_string = "Multi-bit ECC")]
    MultiBitECC,
    #[strum(to_string = "CRC")]
    Crc,
}

impl From<u8> for ErrorCorrection {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Other,
            2 => Self::Unknown,
            3 => Self::None,
            4 => Self::Parity,
            5 => Self::SingleBitECC,
            6 => Self::MultiBitECC,
            7 => Self::Crc,
            _ => unreachable!(),
        }
    }
}
