use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, List, ListItem, ListState, Padding, Row, Table},
};

#[derive(Debug)]
pub struct Memory {
    pub physical_memory_array: PhysicalMemoryArray,
    pub memory_devices: Vec<MemoryDevice>,
    selected_device: usize,
}

impl Memory {
    pub fn new(
        physical_memory_array: PhysicalMemoryArray,
        memory_devices: Vec<MemoryDevice>,
    ) -> Self {
        Self {
            physical_memory_array,
            memory_devices,
            selected_device: 0,
        }
    }

    fn device_layout(&self) -> DeviceLayout {
        let mut has_soldered = false;
        let mut has_socketed = false;
        for d in &self.memory_devices {
            match d.form_factor.kind() {
                FormFactorKind::Soldered => has_soldered = true,
                FormFactorKind::Socketed => has_socketed = true,
                FormFactorKind::Unknown => {}
            }
        }
        match (has_soldered, has_socketed) {
            (true, false) => DeviceLayout::Soldered,
            (false, true) => DeviceLayout::Socketed,
            _ => DeviceLayout::Mixed,
        }
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) {
        if self.memory_devices.is_empty() {
            return;
        }
        match key_event.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected_device = (self.selected_device + 1) % self.memory_devices.len();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_device = (self.selected_device + self.memory_devices.len() - 1)
                    % self.memory_devices.len();
            }
            _ => {}
        }
    }

    pub fn render(&mut self, frame: &mut Frame, block: Rect) {
        if self.memory_devices.is_empty() {
            self.physical_memory_array.render(frame, block);
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Fill(1)])
            .split(block.inner(Margin::new(4, 2)));

        let count_label = match self.device_layout() {
            DeviceLayout::Soldered => "Chips: ",
            DeviceLayout::Socketed => "Slots: ",
            DeviceLayout::Mixed => "Devices: ",
        };

        let summary = Line::from(vec![
            Span::from("Total Capacity: ").bold(),
            Span::from(self.physical_memory_array.max_capacity.clone()),
            Span::from("    "),
            Span::from(count_label).bold(),
            Span::from(self.physical_memory_array.number_memory_devices.to_string()),
            Span::from("    "),
            Span::from("ECC: ").bold(),
            Span::from(self.physical_memory_array.error_correction.to_string()),
        ]);
        frame.render_widget(summary, chunks[0]);

        let max_label = self
            .memory_devices
            .iter()
            .map(|d| d.device_locator.chars().count())
            .max()
            .unwrap_or(0) as u16;
        // 4 = 2 borders + 2 horizontal padding
        let list_width = max_label.saturating_add(4).max(14);

        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(list_width), Constraint::Fill(1)])
            .split(chunks[1]);

        let items: Vec<ListItem<'_>> = self
            .memory_devices
            .iter()
            .map(|d| ListItem::new(d.device_locator.clone()))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .padding(Padding::new(1, 1, 1, 0)),
            )
            .highlight_style(Style::new().bold().reversed())
            .highlight_symbol("");

        let mut state = ListState::default();
        state.select(Some(self.selected_device));
        frame.render_stateful_widget(list, body[0], &mut state);

        if let Some(device) = self.memory_devices.get(self.selected_device) {
            device.render(frame, body[1]);
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum DeviceLayout {
    Soldered,
    Socketed,
    Mixed,
}

#[derive(Debug, Clone, Copy)]
enum FormFactorKind {
    Soldered,
    Socketed,
    Unknown,
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
            // Per SMBIOS spec, 0x80000000 in the DWORD field means the actual
            // value is in the Extended Maximum Capacity QWORD (in bytes).
            let kb: u64 = if value == 0x80000000 && data.len() >= 19 {
                u64::from_le_bytes(data[11..19].try_into().unwrap()) / 1024
            } else {
                value as u64
            };

            if kb <= 1024 {
                format!("{kb}K")
            } else if kb <= 1024 * 1024 {
                format!("{}M", kb / 1024)
            } else if kb <= 1024 * 1024 * 1024 {
                format!("{}G", kb / 1024 / 1024)
            } else {
                format!("{}T", kb / 1024 / 1024 / 1024)
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

fn string_ref(idx: u8, text: &[String]) -> String {
    if idx == 0 {
        return "Not Specified".to_string();
    }
    text.get((idx - 1) as usize)
        .cloned()
        .unwrap_or_else(|| "Not Specified".to_string())
}

#[derive(Debug)]
pub struct MemoryDevice {
    device_locator: String,
    bank_locator: String,
    size: MemorySize,
    form_factor: FormFactor,
    memory_type: MemoryType,
    memory_technology: MemoryTechnology,
    speed: Option<u16>,
    configured_speed: Option<u16>,
    rank: Option<u8>,
    configured_voltage_mv: Option<u16>,
    manufacturer: String,
    serial_number: String,
    asset_tag: String,
    part_number: String,
}

impl From<(Vec<u8>, Vec<String>)> for MemoryDevice {
    fn from((data, text): (Vec<u8>, Vec<String>)) -> Self {
        let size_field = u16::from_le_bytes(data[8..10].try_into().unwrap());
        let extended_size = if data.len() >= 28 {
            Some(u32::from_le_bytes(data[24..28].try_into().unwrap()))
        } else {
            None
        };
        let size = MemorySize::from_fields(size_field, extended_size);

        let form_factor = FormFactor::from(data[10]);
        let memory_type = MemoryType::from(data[14]);

        let speed = {
            let v = u16::from_le_bytes(data[17..19].try_into().unwrap());
            if v == 0 { None } else { Some(v) }
        };

        let rank = data
            .get(23)
            .copied()
            .map(|b| b & 0x0F)
            .filter(|r| *r != 0);

        let configured_speed = data
            .get(28..30)
            .and_then(|s| s.try_into().ok())
            .map(u16::from_le_bytes)
            .filter(|v| *v != 0);

        let configured_voltage_mv = data
            .get(34..36)
            .and_then(|s| s.try_into().ok())
            .map(u16::from_le_bytes)
            .filter(|v| *v != 0);

        let memory_technology = data
            .get(36)
            .copied()
            .map(MemoryTechnology::from)
            .unwrap_or(MemoryTechnology::Unknown);

        let manufacturer = data.get(19).copied().map_or_else(
            || "Not Specified".to_string(),
            |b| string_ref(b, &text),
        );
        let serial_number = data.get(20).copied().map_or_else(
            || "Not Specified".to_string(),
            |b| string_ref(b, &text),
        );
        let asset_tag = data.get(21).copied().map_or_else(
            || "Not Specified".to_string(),
            |b| string_ref(b, &text),
        );
        let part_number = data.get(22).copied().map_or_else(
            || "Not Specified".to_string(),
            |b| string_ref(b, &text),
        );

        Self {
            device_locator: string_ref(data[12], &text),
            bank_locator: string_ref(data[13], &text),
            size,
            form_factor,
            memory_type,
            memory_technology,
            speed,
            configured_speed,
            rank,
            configured_voltage_mv,
            manufacturer,
            serial_number,
            asset_tag,
            part_number,
        }
    }
}

impl MemoryDevice {
    fn render(&self, frame: &mut Frame, block: Rect) {
        let speed_text = match self.speed {
            Some(v) => format!("{v} MT/s"),
            None => "Unknown".to_string(),
        };
        let configured_speed_text = match self.configured_speed {
            Some(v) => format!("{v} MT/s"),
            None => "Unknown".to_string(),
        };
        let rank_text = match self.rank {
            Some(v) => v.to_string(),
            None => "Unknown".to_string(),
        };
        let voltage_text = match self.configured_voltage_mv {
            Some(mv) => format_voltage(mv),
            None => "Unknown".to_string(),
        };

        let rows = vec![
            Row::new(vec![
                Cell::from("Size").bold(),
                Cell::from(self.size.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Type").bold(),
                Cell::from(self.memory_type.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Technology").bold(),
                Cell::from(self.memory_technology.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Form Factor").bold(),
                Cell::from(self.form_factor.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Rank").bold(),
                Cell::from(rank_text),
            ]),
            Row::new(vec![
                Cell::from("Speed").bold(),
                Cell::from(speed_text),
            ]),
            Row::new(vec![
                Cell::from("Configured Speed").bold(),
                Cell::from(configured_speed_text),
            ]),
            Row::new(vec![
                Cell::from("Voltage").bold(),
                Cell::from(voltage_text),
            ]),
            Row::new(vec![
                Cell::from("Bank Locator").bold(),
                Cell::from(self.bank_locator.clone()),
            ]),
            Row::new(vec![
                Cell::from("Manufacturer").bold(),
                Cell::from(self.manufacturer.clone()),
            ]),
            Row::new(vec![
                Cell::from("Part Number").bold(),
                Cell::from(self.part_number.clone()),
            ]),
            Row::new(vec![
                Cell::from("Serial Number").bold(),
                Cell::from(self.serial_number.clone()),
            ]),
            Row::new(vec![
                Cell::from("Asset Tag").bold(),
                Cell::from(self.asset_tag.clone()),
            ]),
        ];

        let widths = [Constraint::Length(18), Constraint::Fill(1)];
        let table = Table::new(rows, widths).block(Block::new().padding(Padding::uniform(2)));
        frame.render_widget(table, block.inner(Margin::new(2, 0)));
    }
}

fn format_voltage(mv: u16) -> String {
    let s = format!("{:.3}", mv as f64 / 1000.0);
    let trimmed = s.trim_end_matches('0').trim_end_matches('.');
    format!("{trimmed} V")
}

#[derive(Debug)]
enum MemorySize {
    Empty,
    Unknown,
    Megabytes(u64),
}

impl MemorySize {
    fn from_fields(size_field: u16, extended_size: Option<u32>) -> Self {
        match size_field {
            0 => MemorySize::Empty,
            0xFFFF => MemorySize::Unknown,
            0x7FFF => match extended_size {
                Some(es) => MemorySize::Megabytes((es & 0x7FFF_FFFF) as u64),
                None => MemorySize::Unknown,
            },
            v if v & 0x8000 == 0 => MemorySize::Megabytes((v & 0x7FFF) as u64),
            v => {
                // KB granularity
                let kb = (v & 0x7FFF) as u64;
                MemorySize::Megabytes(kb / 1024)
            }
        }
    }
}

impl std::fmt::Display for MemorySize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemorySize::Empty => write!(f, "Empty"),
            MemorySize::Unknown => write!(f, "Unknown"),
            MemorySize::Megabytes(mb) => {
                if *mb >= 1024 && mb.is_multiple_of(1024) {
                    write!(f, "{} GB", mb / 1024)
                } else if *mb >= 1024 {
                    write!(f, "{:.1} GB", *mb as f64 / 1024.0)
                } else {
                    write!(f, "{mb} MB")
                }
            }
        }
    }
}

#[derive(Debug, strum::Display)]
enum FormFactor {
    #[strum(to_string = "Other")]
    Other,
    #[strum(to_string = "Unknown")]
    Unknown,
    #[strum(to_string = "SIMM")]
    Simm,
    #[strum(to_string = "SIP")]
    Sip,
    #[strum(to_string = "Chip")]
    Chip,
    #[strum(to_string = "DIP")]
    Dip,
    #[strum(to_string = "ZIP")]
    Zip,
    #[strum(to_string = "Proprietary Card")]
    ProprietaryCard,
    #[strum(to_string = "DIMM")]
    Dimm,
    #[strum(to_string = "TSOP")]
    Tsop,
    #[strum(to_string = "Row of chips")]
    RowOfChips,
    #[strum(to_string = "RIMM")]
    Rimm,
    #[strum(to_string = "SODIMM")]
    Sodimm,
    #[strum(to_string = "SRIMM")]
    Srimm,
    #[strum(to_string = "FB-DIMM")]
    FbDimm,
    #[strum(to_string = "Die")]
    Die,
}

impl From<u8> for FormFactor {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Other,
            3 => Self::Simm,
            4 => Self::Sip,
            5 => Self::Chip,
            6 => Self::Dip,
            7 => Self::Zip,
            8 => Self::ProprietaryCard,
            9 => Self::Dimm,
            10 => Self::Tsop,
            11 => Self::RowOfChips,
            12 => Self::Rimm,
            13 => Self::Sodimm,
            14 => Self::Srimm,
            15 => Self::FbDimm,
            16 => Self::Die,
            _ => Self::Unknown,
        }
    }
}

impl FormFactor {
    fn kind(&self) -> FormFactorKind {
        match self {
            Self::Chip | Self::RowOfChips | Self::Die => FormFactorKind::Soldered,
            Self::Simm
            | Self::Sip
            | Self::Dip
            | Self::Zip
            | Self::ProprietaryCard
            | Self::Dimm
            | Self::Tsop
            | Self::Rimm
            | Self::Sodimm
            | Self::Srimm
            | Self::FbDimm => FormFactorKind::Socketed,
            Self::Other | Self::Unknown => FormFactorKind::Unknown,
        }
    }
}

#[derive(Debug, strum::Display)]
enum MemoryType {
    #[strum(to_string = "Other")]
    Other,
    #[strum(to_string = "Unknown")]
    Unknown,
    #[strum(to_string = "DRAM")]
    Dram,
    #[strum(to_string = "EDRAM")]
    Edram,
    #[strum(to_string = "VRAM")]
    Vram,
    #[strum(to_string = "SRAM")]
    Sram,
    #[strum(to_string = "RAM")]
    Ram,
    #[strum(to_string = "ROM")]
    Rom,
    #[strum(to_string = "FLASH")]
    Flash,
    #[strum(to_string = "EEPROM")]
    Eeprom,
    #[strum(to_string = "FEPROM")]
    Feprom,
    #[strum(to_string = "EPROM")]
    Eprom,
    #[strum(to_string = "CDRAM")]
    Cdram,
    #[strum(to_string = "3DRAM")]
    Dram3D,
    #[strum(to_string = "SDRAM")]
    Sdram,
    #[strum(to_string = "SGRAM")]
    Sgram,
    #[strum(to_string = "RDRAM")]
    Rdram,
    #[strum(to_string = "DDR")]
    Ddr,
    #[strum(to_string = "DDR2")]
    Ddr2,
    #[strum(to_string = "DDR2 FB-DIMM")]
    Ddr2FbDimm,
    #[strum(to_string = "DDR3")]
    Ddr3,
    #[strum(to_string = "FBD2")]
    Fbd2,
    #[strum(to_string = "DDR4")]
    Ddr4,
    #[strum(to_string = "LPDDR")]
    LpDdr,
    #[strum(to_string = "LPDDR2")]
    LpDdr2,
    #[strum(to_string = "LPDDR3")]
    LpDdr3,
    #[strum(to_string = "LPDDR4")]
    LpDdr4,
    #[strum(to_string = "Logical non-volatile device")]
    LogicalNonVolatile,
    #[strum(to_string = "HBM")]
    Hbm,
    #[strum(to_string = "HBM2")]
    Hbm2,
    #[strum(to_string = "DDR5")]
    Ddr5,
    #[strum(to_string = "LPDDR5")]
    LpDdr5,
    #[strum(to_string = "HBM3")]
    Hbm3,
}

#[derive(Debug, strum::Display)]
enum MemoryTechnology {
    #[strum(to_string = "Other")]
    Other,
    #[strum(to_string = "Unknown")]
    Unknown,
    #[strum(to_string = "DRAM")]
    Dram,
    #[strum(to_string = "NVDIMM-N")]
    NvdimmN,
    #[strum(to_string = "NVDIMM-F")]
    NvdimmF,
    #[strum(to_string = "NVDIMM-P")]
    NvdimmP,
    #[strum(to_string = "Intel Optane persistent memory")]
    IntelOptane,
}

impl From<u8> for MemoryTechnology {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Other,
            3 => Self::Dram,
            4 => Self::NvdimmN,
            5 => Self::NvdimmF,
            6 => Self::NvdimmP,
            7 => Self::IntelOptane,
            _ => Self::Unknown,
        }
    }
}

impl From<u8> for MemoryType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Other,
            3 => Self::Dram,
            4 => Self::Edram,
            5 => Self::Vram,
            6 => Self::Sram,
            7 => Self::Ram,
            8 => Self::Rom,
            9 => Self::Flash,
            10 => Self::Eeprom,
            11 => Self::Feprom,
            12 => Self::Eprom,
            13 => Self::Cdram,
            14 => Self::Dram3D,
            15 => Self::Sdram,
            16 => Self::Sgram,
            17 => Self::Rdram,
            18 => Self::Ddr,
            19 => Self::Ddr2,
            20 => Self::Ddr2FbDimm,
            24 => Self::Ddr3,
            25 => Self::Fbd2,
            26 => Self::Ddr4,
            27 => Self::LpDdr,
            28 => Self::LpDdr2,
            29 => Self::LpDdr3,
            30 => Self::LpDdr4,
            31 => Self::LogicalNonVolatile,
            32 => Self::Hbm,
            33 => Self::Hbm2,
            34 => Self::Ddr5,
            35 => Self::LpDdr5,
            36 => Self::Hbm3,
            _ => Self::Unknown,
        }
    }
}
