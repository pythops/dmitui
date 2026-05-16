use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Style, Stylize},
    widgets::{Block, BorderType, Borders, Cell, List, ListItem, ListState, Padding, Row, Table},
};

use crate::dmi::cache::Cache;

fn string_ref(idx: u8, text: &[String]) -> String {
    if idx == 0 {
        return "Not Specified".to_string();
    }
    text.get((idx - 1) as usize)
        .cloned()
        .unwrap_or_else(|| "Not Specified".to_string())
}

#[derive(Debug)]
pub struct Processors {
    list: Vec<Processor>,
    caches: Vec<Cache>,
    selected: usize,
}

impl Processors {
    pub fn new(list: Vec<Processor>, caches: Vec<Cache>) -> Option<Self> {
        if list.is_empty() {
            None
        } else {
            Some(Self {
                list,
                caches,
                selected: 0,
            })
        }
    }

    pub fn has_multiple(&self) -> bool {
        self.list.len() >= 2
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) {
        if !self.has_multiple() {
            return;
        }
        match key_event.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = (self.selected + 1) % self.list.len();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = (self.selected + self.list.len() - 1) % self.list.len();
            }
            _ => {}
        }
    }

    pub fn render(&mut self, frame: &mut Frame, block: Rect) {
        if !self.has_multiple() {
            self.list[0].render(frame, block, &self.caches);
            return;
        }

        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(22), Constraint::Fill(1)])
            .split(block.inner(Margin::new(2, 1)));

        let items: Vec<ListItem<'_>> = self
            .list
            .iter()
            .map(|p| ListItem::new(p.socket_designation.clone()))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .padding(Padding::horizontal(1)),
            )
            .highlight_style(Style::new().bold().reversed())
            .highlight_symbol("");

        let mut state = ListState::default();
        state.select(Some(self.selected));
        frame.render_stateful_widget(list, body[0], &mut state);

        if let Some(processor) = self.list.get(self.selected) {
            processor.render(frame, body[1], &self.caches);
        }
    }
}

#[derive(Debug)]
pub struct Processor {
    socket_designation: String,
    processor_type: ProcessorType,
    family: u16,
    manufacturer: String,
    version: String,
    voltage: VoltageInfo,
    max_speed: Option<u16>,
    current_speed: Option<u16>,
    status: ProcessorStatus,
    upgrade: u8,
    l1_cache: Option<u16>,
    l2_cache: Option<u16>,
    l3_cache: Option<u16>,
    core_count: Option<u16>,
    core_enabled: Option<u16>,
    thread_count: Option<u16>,
    serial_number: String,
    asset_tag: String,
    part_number: String,
}

impl From<(Vec<u8>, Vec<String>)> for Processor {
    fn from((data, text): (Vec<u8>, Vec<String>)) -> Self {
        let family = {
            let f1 = data[2];
            if f1 == 0xFE && data.len() >= 38 {
                u16::from_le_bytes(data[36..38].try_into().unwrap())
            } else {
                f1 as u16
            }
        };

        let max_speed = {
            let v = u16::from_le_bytes(data[16..18].try_into().unwrap());
            (v != 0).then_some(v)
        };
        let current_speed = {
            let v = u16::from_le_bytes(data[18..20].try_into().unwrap());
            (v != 0).then_some(v)
        };

        let core_count = read_count(data.get(31).copied(), data.get(38..40));
        let core_enabled = read_count(data.get(32).copied(), data.get(40..42));
        let thread_count = read_count(data.get(33).copied(), data.get(42..44));

        let serial_number = data.get(28).copied().map_or_else(
            || "Not Specified".to_string(),
            |b| string_ref(b, &text),
        );
        let asset_tag = data.get(29).copied().map_or_else(
            || "Not Specified".to_string(),
            |b| string_ref(b, &text),
        );
        let part_number = data.get(30).copied().map_or_else(
            || "Not Specified".to_string(),
            |b| string_ref(b, &text),
        );

        let l1_cache = cache_handle(&data, 22);
        let l2_cache = cache_handle(&data, 24);
        let l3_cache = cache_handle(&data, 26);

        Self {
            socket_designation: string_ref(data[0], &text),
            processor_type: ProcessorType::from(data[1]),
            family,
            manufacturer: string_ref(data[3], &text),
            version: string_ref(data[12], &text),
            voltage: VoltageInfo::from(data[13]),
            max_speed,
            current_speed,
            status: ProcessorStatus::from(data[20]),
            upgrade: data[21],
            l1_cache,
            l2_cache,
            l3_cache,
            core_count,
            core_enabled,
            thread_count,
            serial_number,
            asset_tag,
            part_number,
        }
    }
}

// Read a u16 cache handle at the given offset in the structure's data slice.
// Returns None if the structure is too short or the handle is 0xFFFF
// ("the device does not have any cache of this level").
fn cache_handle(data: &[u8], offset: usize) -> Option<u16> {
    let slice = data.get(offset..offset + 2)?;
    let handle = u16::from_le_bytes(slice.try_into().ok()?);
    (handle != 0xFFFF).then_some(handle)
}

fn read_count(legacy: Option<u8>, extended: Option<&[u8]>) -> Option<u16> {
    match legacy? {
        0 => None,
        0xFF => extended
            .and_then(|s| s.try_into().ok())
            .map(u16::from_le_bytes),
        v => Some(v as u16),
    }
}

impl Processor {
    fn render(&self, frame: &mut Frame, block: Rect, caches: &[Cache]) {
        let speed_cell = |v: Option<u16>| match v {
            Some(s) => format!("{s} MHz"),
            None => "Unknown".to_string(),
        };
        let count_cell = |v: Option<u16>| match v {
            Some(c) => c.to_string(),
            None => "Unknown".to_string(),
        };
        let cache_row = |label: &'static str, handle: Option<u16>| {
            let summary = handle
                .and_then(|h| caches.iter().find(|c| c.handle == h))
                .map(Cache::summary)
                .unwrap_or_else(|| "Not present".to_string());
            Row::new(vec![Cell::from(label).bold(), Cell::from(summary)])
        };

        let rows = vec![
            Row::new(vec![
                Cell::from("Socket").bold(),
                Cell::from(self.socket_designation.clone()),
            ]),
            Row::new(vec![
                Cell::from("Type").bold(),
                Cell::from(self.processor_type.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Manufacturer").bold(),
                Cell::from(self.manufacturer.clone()),
            ]),
            Row::new(vec![
                Cell::from("Version").bold(),
                Cell::from(self.version.clone()),
            ]),
            Row::new(vec![
                Cell::from("Family").bold(),
                Cell::from(family_name(self.family, &self.manufacturer)),
            ]),
            Row::new(vec![
                Cell::from("Max Speed").bold(),
                Cell::from(speed_cell(self.max_speed)),
            ]),
            Row::new(vec![
                Cell::from("Current Speed").bold(),
                Cell::from(speed_cell(self.current_speed)),
            ]),
            Row::new(vec![
                Cell::from("Cores").bold(),
                Cell::from(count_cell(self.core_count)),
            ]),
            Row::new(vec![
                Cell::from("Cores Enabled").bold(),
                Cell::from(count_cell(self.core_enabled)),
            ]),
            Row::new(vec![
                Cell::from("Threads").bold(),
                Cell::from(count_cell(self.thread_count)),
            ]),
            Row::new(vec![
                Cell::from("Voltage").bold(),
                Cell::from(self.voltage.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Status").bold(),
                Cell::from(self.status.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Upgrade").bold(),
                Cell::from(upgrade_name(self.upgrade)),
            ]),
            cache_row("L1 Cache", self.l1_cache),
            cache_row("L2 Cache", self.l2_cache),
            cache_row("L3 Cache", self.l3_cache),
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
        let table = Table::new(rows, widths).block(Block::new().padding(Padding::uniform(1)));
        frame.render_widget(table, block.inner(Margin::new(2, 0)));
    }
}

#[derive(Debug, strum::Display)]
enum ProcessorType {
    #[strum(to_string = "Other")]
    Other,
    #[strum(to_string = "Unknown")]
    Unknown,
    #[strum(to_string = "Central Processor")]
    Central,
    #[strum(to_string = "Math Processor")]
    Math,
    #[strum(to_string = "DSP Processor")]
    Dsp,
    #[strum(to_string = "Video Processor")]
    Video,
}

impl From<u8> for ProcessorType {
    fn from(value: u8) -> Self {
        match value {
            3 => Self::Central,
            4 => Self::Math,
            5 => Self::Dsp,
            6 => Self::Video,
            2 => Self::Unknown,
            _ => Self::Other,
        }
    }
}

#[derive(Debug)]
struct ProcessorStatus {
    populated: bool,
    cpu_status: u8,
}

impl From<u8> for ProcessorStatus {
    fn from(value: u8) -> Self {
        Self {
            populated: (value & 0b0100_0000) != 0,
            cpu_status: value & 0b0000_0111,
        }
    }
}

impl std::fmt::Display for ProcessorStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.populated {
            return write!(f, "Unpopulated");
        }
        let cpu = match self.cpu_status {
            0 => "Unknown",
            1 => "Enabled",
            2 => "Disabled by user (BIOS Setup)",
            3 => "Disabled by BIOS (POST Error)",
            4 => "Idle, waiting to be enabled",
            7 => "Other",
            _ => "Reserved",
        };
        write!(f, "Populated, {cpu}")
    }
}

#[derive(Debug)]
enum VoltageInfo {
    Current(u8), // tenths of a volt
    Capabilities { v5: bool, v33: bool, v29: bool },
}

impl From<u8> for VoltageInfo {
    fn from(value: u8) -> Self {
        if value & 0b1000_0000 != 0 {
            Self::Current(value & 0b0111_1111)
        } else {
            Self::Capabilities {
                v5: value & 0b0000_0001 != 0,
                v33: value & 0b0000_0010 != 0,
                v29: value & 0b0000_0100 != 0,
            }
        }
    }
}

impl std::fmt::Display for VoltageInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Current(tenths) => {
                let volts = *tenths as f64 / 10.0;
                write!(f, "{volts:.1} V")
            }
            Self::Capabilities { v5, v33, v29 } => {
                let mut parts: Vec<&str> = Vec::new();
                if *v5 {
                    parts.push("5 V");
                }
                if *v33 {
                    parts.push("3.3 V");
                }
                if *v29 {
                    parts.push("2.9 V");
                }
                if parts.is_empty() {
                    write!(f, "Unknown")
                } else {
                    write!(f, "Capable of {}", parts.join(", "))
                }
            }
        }
    }
}

// Family table transcribed from dmidecode 3.7+ (dmi_processor_family in dmidecode.c).
// Spec reference: SMBIOS DSP0134 §7.5.2.
const FAMILY_NAMES: &[(u16, &str)] = &[
    (0x01, "Other"),
    (0x02, "Unknown"),
    (0x03, "8086"),
    (0x04, "80286"),
    (0x05, "80386"),
    (0x06, "80486"),
    (0x07, "8087"),
    (0x08, "80287"),
    (0x09, "80387"),
    (0x0A, "80487"),
    (0x0B, "Pentium"),
    (0x0C, "Pentium Pro"),
    (0x0D, "Pentium II"),
    (0x0E, "Pentium MMX"),
    (0x0F, "Celeron"),
    (0x10, "Pentium II Xeon"),
    (0x11, "Pentium III"),
    (0x12, "M1"),
    (0x13, "M2"),
    (0x14, "Celeron M"),
    (0x15, "Pentium 4 HT"),
    (0x16, "Intel"),
    (0x18, "Duron"),
    (0x19, "K5"),
    (0x1A, "K6"),
    (0x1B, "K6-2"),
    (0x1C, "K6-3"),
    (0x1D, "Athlon"),
    (0x1E, "AMD29000"),
    (0x1F, "K6-2+"),
    (0x20, "Power PC"),
    (0x21, "Power PC 601"),
    (0x22, "Power PC 603"),
    (0x23, "Power PC 603+"),
    (0x24, "Power PC 604"),
    (0x25, "Power PC 620"),
    (0x26, "Power PC x704"),
    (0x27, "Power PC 750"),
    (0x28, "Core Duo"),
    (0x29, "Core Duo Mobile"),
    (0x2A, "Core Solo Mobile"),
    (0x2B, "Atom"),
    (0x2C, "Core M"),
    (0x2D, "Core m3"),
    (0x2E, "Core m5"),
    (0x2F, "Core m7"),
    (0x30, "Alpha"),
    (0x31, "Alpha 21064"),
    (0x32, "Alpha 21066"),
    (0x33, "Alpha 21164"),
    (0x34, "Alpha 21164PC"),
    (0x35, "Alpha 21164a"),
    (0x36, "Alpha 21264"),
    (0x37, "Alpha 21364"),
    (0x38, "Turion II Ultra Dual-Core Mobile M"),
    (0x39, "Turion II Dual-Core Mobile M"),
    (0x3A, "Athlon II Dual-Core M"),
    (0x3B, "Opteron 6100"),
    (0x3C, "Opteron 4100"),
    (0x3D, "Opteron 6200"),
    (0x3E, "Opteron 4200"),
    (0x3F, "FX"),
    (0x40, "MIPS"),
    (0x41, "MIPS R4000"),
    (0x42, "MIPS R4200"),
    (0x43, "MIPS R4400"),
    (0x44, "MIPS R4600"),
    (0x45, "MIPS R10000"),
    (0x46, "C-Series"),
    (0x47, "E-Series"),
    (0x48, "A-Series"),
    (0x49, "G-Series"),
    (0x4A, "Z-Series"),
    (0x4B, "R-Series"),
    (0x4C, "Opteron 4300"),
    (0x4D, "Opteron 6300"),
    (0x4E, "Opteron 3300"),
    (0x4F, "FirePro"),
    (0x50, "SPARC"),
    (0x51, "SuperSPARC"),
    (0x52, "MicroSPARC II"),
    (0x53, "MicroSPARC IIep"),
    (0x54, "UltraSPARC"),
    (0x55, "UltraSPARC II"),
    (0x56, "UltraSPARC IIi"),
    (0x57, "UltraSPARC III"),
    (0x58, "UltraSPARC IIIi"),
    (0x60, "68040"),
    (0x61, "68xxx"),
    (0x62, "68000"),
    (0x63, "68010"),
    (0x64, "68020"),
    (0x65, "68030"),
    (0x66, "Athlon X4"),
    (0x67, "Opteron X1000"),
    (0x68, "Opteron X2000"),
    (0x69, "Opteron A-Series"),
    (0x6A, "Opteron X3000"),
    (0x6B, "Zen"),
    (0x70, "Hobbit"),
    (0x78, "Crusoe TM5000"),
    (0x79, "Crusoe TM3000"),
    (0x7A, "Efficeon TM8000"),
    (0x80, "Weitek"),
    (0x82, "Itanium"),
    (0x83, "Athlon 64"),
    (0x84, "Opteron"),
    (0x85, "Sempron"),
    (0x86, "Turion 64"),
    (0x87, "Dual-Core Opteron"),
    (0x88, "Athlon 64 X2"),
    (0x89, "Turion 64 X2"),
    (0x8A, "Quad-Core Opteron"),
    (0x8B, "Third-Generation Opteron"),
    (0x8C, "Phenom FX"),
    (0x8D, "Phenom X4"),
    (0x8E, "Phenom X2"),
    (0x8F, "Athlon X2"),
    (0x90, "PA-RISC"),
    (0x91, "PA-RISC 8500"),
    (0x92, "PA-RISC 8000"),
    (0x93, "PA-RISC 7300LC"),
    (0x94, "PA-RISC 7200"),
    (0x95, "PA-RISC 7100LC"),
    (0x96, "PA-RISC 7100"),
    (0xA0, "V30"),
    (0xA1, "Quad-Core Xeon 3200"),
    (0xA2, "Dual-Core Xeon 3000"),
    (0xA3, "Quad-Core Xeon 5300"),
    (0xA4, "Dual-Core Xeon 5100"),
    (0xA5, "Dual-Core Xeon 5000"),
    (0xA6, "Dual-Core Xeon LV"),
    (0xA7, "Dual-Core Xeon ULV"),
    (0xA8, "Dual-Core Xeon 7100"),
    (0xA9, "Quad-Core Xeon 5400"),
    (0xAA, "Quad-Core Xeon"),
    (0xAB, "Dual-Core Xeon 5200"),
    (0xAC, "Dual-Core Xeon 7200"),
    (0xAD, "Quad-Core Xeon 7300"),
    (0xAE, "Quad-Core Xeon 7400"),
    (0xAF, "Multi-Core Xeon 7400"),
    (0xB0, "Pentium III Xeon"),
    (0xB1, "Pentium III Speedstep"),
    (0xB2, "Pentium 4"),
    (0xB3, "Xeon"),
    (0xB4, "AS400"),
    (0xB5, "Xeon MP"),
    (0xB6, "Athlon XP"),
    (0xB7, "Athlon MP"),
    (0xB8, "Itanium 2"),
    (0xB9, "Pentium M"),
    (0xBA, "Celeron D"),
    (0xBB, "Pentium D"),
    (0xBC, "Pentium EE"),
    (0xBD, "Core Solo"),
    // 0xBE handled as special case in family_name
    (0xBF, "Core 2 Duo"),
    (0xC0, "Core 2 Solo"),
    (0xC1, "Core 2 Extreme"),
    (0xC2, "Core 2 Quad"),
    (0xC3, "Core 2 Extreme Mobile"),
    (0xC4, "Core 2 Duo Mobile"),
    (0xC5, "Core 2 Solo Mobile"),
    (0xC6, "Core i7"),
    (0xC7, "Dual-Core Celeron"),
    (0xC8, "IBM390"),
    (0xC9, "G4"),
    (0xCA, "G5"),
    (0xCB, "ESA/390 G6"),
    (0xCC, "z/Architecture"),
    (0xCD, "Core i5"),
    (0xCE, "Core i3"),
    (0xCF, "Core i9"),
    (0xD2, "C7-M"),
    (0xD3, "C7-D"),
    (0xD4, "C7"),
    (0xD5, "Eden"),
    (0xD6, "Multi-Core Xeon"),
    (0xD7, "Dual-Core Xeon 3xxx"),
    (0xD8, "Quad-Core Xeon 3xxx"),
    (0xD9, "Nano"),
    (0xDA, "Dual-Core Xeon 5xxx"),
    (0xDB, "Quad-Core Xeon 5xxx"),
    (0xDD, "Dual-Core Xeon 7xxx"),
    (0xDE, "Quad-Core Xeon 7xxx"),
    (0xDF, "Multi-Core Xeon 7xxx"),
    (0xE0, "Multi-Core Xeon 3400"),
    (0xE4, "Opteron 3000"),
    (0xE5, "Sempron II"),
    (0xE6, "Embedded Opteron Quad-Core"),
    (0xE7, "Phenom Triple-Core"),
    (0xE8, "Turion Ultra Dual-Core Mobile"),
    (0xE9, "Turion Dual-Core Mobile"),
    (0xEA, "Athlon Dual-Core"),
    (0xEB, "Sempron SI"),
    (0xEC, "Phenom II"),
    (0xED, "Athlon II"),
    (0xEE, "Six-Core Opteron"),
    (0xEF, "Sempron M"),
    (0xFA, "i860"),
    (0xFB, "i960"),
    (0x100, "ARMv7"),
    (0x101, "ARMv8"),
    (0x102, "ARMv9"),
    (0x103, "ARM"),
    (0x104, "SH-3"),
    (0x105, "SH-4"),
    (0x118, "ARM"),
    (0x119, "StrongARM"),
    (0x12C, "6x86"),
    (0x12D, "MediaGX"),
    (0x12E, "MII"),
    (0x140, "WinChip"),
    (0x15E, "DSP"),
    (0x1F4, "Video Processor"),
    (0x200, "RV32"),
    (0x201, "RV64"),
    (0x202, "RV128"),
    (0x258, "LoongArch"),
    (0x259, "Loongson 1"),
    (0x25A, "Loongson 2"),
    (0x25B, "Loongson 3"),
    (0x25C, "Loongson 2K"),
    (0x25D, "Loongson 3A"),
    (0x25E, "Loongson 3B"),
    (0x25F, "Loongson 3C"),
    (0x260, "Loongson 3D"),
    (0x261, "Loongson 3E"),
    (0x262, "Dual-Core Loongson 2K 2xxx"),
    (0x26C, "Quad-Core Loongson 3A 5xxx"),
    (0x26D, "Multi-Core Loongson 3A 5xxx"),
    (0x26E, "Quad-Core Loongson 3B 5xxx"),
    (0x26F, "Multi-Core Loongson 3B 5xxx"),
    (0x270, "Multi-Core Loongson 3C 5xxx"),
    (0x271, "Multi-Core Loongson 3D 5xxx"),
];

fn family_name(family: u16, manufacturer: &str) -> String {
    // 0xBE is ambiguous: Intel Core 2 vs AMD K7. Decode using manufacturer.
    if family == 0xBE {
        if manufacturer.contains("Intel") {
            return "Core 2".to_string();
        }
        if manufacturer.contains("AMD") {
            return "K7".to_string();
        }
        return "Core 2 or K7".to_string();
    }

    FAMILY_NAMES
        .iter()
        .find_map(|(id, name)| (*id == family).then_some((*name).to_string()))
        .unwrap_or_else(|| format!("Family {family:#x}"))
}

// Upgrade array transcribed from dmidecode 3.7+ (dmi_processor_upgrade in dmidecode.c).
// Spec reference: SMBIOS DSP0134 §7.5.5. Indexed by code - 0x01.
const UPGRADE_NAMES: &[&str] = &[
    "Other",                 // 0x01
    "Unknown",               // 0x02
    "Daughter Board",        // 0x03
    "ZIF Socket",            // 0x04
    "Replaceable Piggy Back",// 0x05
    "None",                  // 0x06
    "LIF Socket",            // 0x07
    "Slot 1",                // 0x08
    "Slot 2",                // 0x09
    "370-pin Socket",        // 0x0A
    "Slot A",                // 0x0B
    "Slot M",                // 0x0C
    "Socket 423",            // 0x0D
    "Socket A (Socket 462)", // 0x0E
    "Socket 478",            // 0x0F
    "Socket 754",            // 0x10
    "Socket 940",            // 0x11
    "Socket 939",            // 0x12
    "Socket mPGA604",        // 0x13
    "Socket LGA771",         // 0x14
    "Socket LGA775",         // 0x15
    "Socket S1",             // 0x16
    "Socket AM2",            // 0x17
    "Socket F (1207)",       // 0x18
    "Socket LGA1366",        // 0x19
    "Socket G34",            // 0x1A
    "Socket AM3",            // 0x1B
    "Socket C32",            // 0x1C
    "Socket LGA1156",        // 0x1D
    "Socket LGA1567",        // 0x1E
    "Socket PGA988A",        // 0x1F
    "Socket BGA1288",        // 0x20
    "Socket rPGA988B",       // 0x21
    "Socket BGA1023",        // 0x22
    "Socket BGA1224",        // 0x23
    "Socket BGA1155",        // 0x24
    "Socket LGA1356",        // 0x25
    "Socket LGA2011",        // 0x26
    "Socket FS1",            // 0x27
    "Socket FS2",            // 0x28
    "Socket FM1",            // 0x29
    "Socket FM2",            // 0x2A
    "Socket LGA2011-3",      // 0x2B
    "Socket LGA1356-3",      // 0x2C
    "Socket LGA1150",        // 0x2D
    "Socket BGA1168",        // 0x2E
    "Socket BGA1234",        // 0x2F
    "Socket BGA1364",        // 0x30
    "Socket AM4",            // 0x31
    "Socket LGA1151",        // 0x32
    "Socket BGA1356",        // 0x33
    "Socket BGA1440",        // 0x34
    "Socket BGA1515",        // 0x35
    "Socket LGA3647-1",      // 0x36
    "Socket SP3",            // 0x37
    "Socket SP3r2",          // 0x38
    "Socket LGA2066",        // 0x39
    "Socket BGA1392",        // 0x3A
    "Socket BGA1510",        // 0x3B
    "Socket BGA1528",        // 0x3C
    "Socket LGA4189",        // 0x3D
    "Socket LGA1200",        // 0x3E
    "Socket LGA4677",        // 0x3F
    "Socket LGA1700",        // 0x40
    "Socket BGA1744",        // 0x41
    "Socket BGA1781",        // 0x42
    "Socket BGA1211",        // 0x43
    "Socket BGA2422",        // 0x44
    "Socket LGA1211",        // 0x45
    "Socket LGA2422",        // 0x46
    "Socket LGA5773",        // 0x47
    "Socket BGA5773",        // 0x48
    "Socket AM5",            // 0x49
    "Socket SP5",            // 0x4A
    "Socket SP6",            // 0x4B
    "Socket BGA883",         // 0x4C
    "Socket BGA1190",        // 0x4D
    "Socket BGA4129",        // 0x4E
    "Socket LGA4710",        // 0x4F
    "Socket LGA7529",        // 0x50
];

fn upgrade_name(upgrade: u8) -> String {
    if !(0x01..=0x50).contains(&upgrade) {
        return format!("Upgrade {upgrade:#x}");
    }
    UPGRADE_NAMES[(upgrade - 1) as usize].to_string()
}
