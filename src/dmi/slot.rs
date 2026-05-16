// SMBIOS Type 9 (System Slots). Spec reference: DSP0134 §7.10.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Style, Stylize},
    widgets::{Block, BorderType, Borders, Cell, List, ListItem, ListState, Padding, Row, Table},
};

fn string_ref(idx: u8, text: &[String]) -> String {
    if idx == 0 {
        return "Not Specified".to_string();
    }
    text.get((idx - 1) as usize)
        .cloned()
        .unwrap_or_else(|| "Not Specified".to_string())
}

#[derive(Debug)]
pub struct Slots {
    list: Vec<Slot>,
    selected: usize,
}

impl Slots {
    pub fn new(list: Vec<Slot>) -> Option<Self> {
        if list.is_empty() {
            None
        } else {
            Some(Self { list, selected: 0 })
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
            self.list[0].render(frame, block);
            return;
        }

        let max_label = self
            .list
            .iter()
            .map(|s| s.designation.chars().count())
            .max()
            .unwrap_or(0) as u16;
        // 4 = 2 borders + 2 horizontal padding
        let list_width = max_label.saturating_add(4).max(14);

        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(list_width), Constraint::Fill(1)])
            .split(block.inner(Margin::new(4, 2)));

        let items: Vec<ListItem<'_>> = self
            .list
            .iter()
            .map(|s| ListItem::new(s.designation.clone()))
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
        state.select(Some(self.selected));
        frame.render_stateful_widget(list, body[0], &mut state);

        if let Some(slot) = self.list.get(self.selected) {
            slot.render(frame, body[1]);
        }
    }
}

#[derive(Debug)]
pub struct Slot {
    designation: String,
    slot_type: u8,
    bus_width: u8,
    current_usage: u8,
    length: u8,
    id: u16,
    bdf: Option<BusDeviceFunction>,
}

#[derive(Debug)]
struct BusDeviceFunction {
    segment: u16,
    bus: u8,
    device: u8,
    function: u8,
}

impl std::fmt::Display for BusDeviceFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:04x}:{:02x}:{:02x}.{}",
            self.segment, self.bus, self.device, self.function
        )
    }
}

impl From<(Vec<u8>, Vec<String>)> for Slot {
    fn from((data, text): (Vec<u8>, Vec<String>)) -> Self {
        let id = u16::from_le_bytes(data[5..7].try_into().unwrap());

        // Segment/Bus/Device-Function only present in SMBIOS 2.6+
        let bdf = if data.len() >= 13 {
            let segment = u16::from_le_bytes(data[9..11].try_into().unwrap());
            let bus = data[11];
            let devfunc = data[12];
            // Unset values are 0xFFFF/0xFF — skip if all unset.
            if segment == 0xFFFF && bus == 0xFF && devfunc == 0xFF {
                None
            } else {
                Some(BusDeviceFunction {
                    segment,
                    bus,
                    device: devfunc >> 3,
                    function: devfunc & 0x07,
                })
            }
        } else {
            None
        };

        Self {
            designation: string_ref(data[0], &text),
            slot_type: data[1],
            bus_width: data[2],
            current_usage: data[3],
            length: data[4],
            id,
            bdf,
        }
    }
}

impl Slot {
    fn render(&self, frame: &mut Frame, block: Rect) {
        let mut rows = vec![
            Row::new(vec![
                Cell::from("Designation").bold(),
                Cell::from(self.designation.clone()),
            ]),
            Row::new(vec![
                Cell::from("Type").bold(),
                Cell::from(slot_type_name(self.slot_type)),
            ]),
            Row::new(vec![
                Cell::from("Bus Width").bold(),
                Cell::from(slot_bus_width_name(self.bus_width)),
            ]),
            Row::new(vec![
                Cell::from("Current Usage").bold(),
                Cell::from(slot_current_usage_name(self.current_usage)),
            ]),
            Row::new(vec![
                Cell::from("Length").bold(),
                Cell::from(slot_length_name(self.length)),
            ]),
            Row::new(vec![Cell::from("ID").bold(), Cell::from(self.id.to_string())]),
        ];
        if let Some(bdf) = &self.bdf {
            rows.push(Row::new(vec![
                Cell::from("Bus:Device.Function").bold(),
                Cell::from(bdf.to_string()),
            ]));
        }

        let widths = [Constraint::Length(22), Constraint::Fill(1)];
        let table = Table::new(rows, widths).block(Block::new().padding(Padding::uniform(2)));
        frame.render_widget(table, block.inner(Margin::new(2, 0)));
    }
}

// Slot Type table transcribed from dmidecode 3.7+ (dmi_slot_type in dmidecode.c).
// Spec reference: SMBIOS DSP0134 §7.10.1.
const SLOT_TYPE_LOW: &[&str] = &[
    "Other",                                              // 0x01
    "Unknown",                                            // 0x02
    "ISA",                                                // 0x03
    "MCA",                                                // 0x04
    "EISA",                                               // 0x05
    "PCI",                                                // 0x06
    "PC Card (PCMCIA)",                                   // 0x07
    "VLB",                                                // 0x08
    "Proprietary",                                        // 0x09
    "Processor Card",                                     // 0x0A
    "Proprietary Memory Card",                            // 0x0B
    "I/O Riser Card",                                     // 0x0C
    "NuBus",                                              // 0x0D
    "PCI-66",                                             // 0x0E
    "AGP",                                                // 0x0F
    "AGP 2x",                                             // 0x10
    "AGP 4x",                                             // 0x11
    "PCI-X",                                              // 0x12
    "AGP 8x",                                             // 0x13
    "M.2 Socket 1-DP",                                    // 0x14
    "M.2 Socket 1-SD",                                    // 0x15
    "M.2 Socket 2",                                       // 0x16
    "M.2 Socket 3",                                       // 0x17
    "MXM Type I",                                         // 0x18
    "MXM Type II",                                        // 0x19
    "MXM Type III",                                       // 0x1A
    "MXM Type III-HE",                                    // 0x1B
    "MXM Type IV",                                        // 0x1C
    "MXM 3.0 Type A",                                     // 0x1D
    "MXM 3.0 Type B",                                     // 0x1E
    "PCI Express 2 SFF-8639 (U.2)",                       // 0x1F
    "PCI Express 3 SFF-8639 (U.2)",                       // 0x20
    "PCI Express Mini 52-pin with bottom-side keep-outs", // 0x21
    "PCI Express Mini 52-pin without bottom-side keep-outs", // 0x22
    "PCI Express Mini 76-pin",                            // 0x23
    "PCI Express 4 SFF-8639 (U.2)",                       // 0x24
    "PCI Express 5 SFF-8639 (U.2)",                       // 0x25
    "OCP NIC 3.0 Small Form Factor (SFF)",                // 0x26
    "OCP NIC 3.0 Large Form Factor (LFF)",                // 0x27
    "OCP NIC Prior to 3.0",                               // 0x28
];

// Mirrors dmidecode's spelling, including "FLexbus".
const SLOT_TYPE_CXL: &str = "CXL FLexbus 1.0";

const SLOT_TYPE_HIGH: &[&str] = &[
    "PC-98/C20",            // 0xA0
    "PC-98/C24",            // 0xA1
    "PC-98/E",              // 0xA2
    "PC-98/Local Bus",      // 0xA3
    "PC-98/Card",           // 0xA4
    "PCI Express",          // 0xA5
    "PCI Express x1",       // 0xA6
    "PCI Express x2",       // 0xA7
    "PCI Express x4",       // 0xA8
    "PCI Express x8",       // 0xA9
    "PCI Express x16",      // 0xAA
    "PCI Express 2",        // 0xAB
    "PCI Express 2 x1",     // 0xAC
    "PCI Express 2 x2",     // 0xAD
    "PCI Express 2 x4",     // 0xAE
    "PCI Express 2 x8",     // 0xAF
    "PCI Express 2 x16",    // 0xB0
    "PCI Express 3",        // 0xB1
    "PCI Express 3 x1",     // 0xB2
    "PCI Express 3 x2",     // 0xB3
    "PCI Express 3 x4",     // 0xB4
    "PCI Express 3 x8",     // 0xB5
    "PCI Express 3 x16",    // 0xB6
    "",                     // 0xB7 — out of spec gap in dmidecode
    "PCI Express 4",        // 0xB8
    "PCI Express 4 x1",     // 0xB9
    "PCI Express 4 x2",     // 0xBA
    "PCI Express 4 x4",     // 0xBB
    "PCI Express 4 x8",     // 0xBC
    "PCI Express 4 x16",    // 0xBD
    "PCI Express 5",        // 0xBE
    "PCI Express 5 x1",     // 0xBF
    "PCI Express 5 x2",     // 0xC0
    "PCI Express 5 x4",     // 0xC1
    "PCI Express 5 x8",     // 0xC2
    "PCI Express 5 x16",    // 0xC3
    "PCI Express 6+",       // 0xC4
    "EDSFF E1",             // 0xC5
    "EDSFF E3",             // 0xC6
];

fn slot_type_name(code: u8) -> String {
    if (0x01..=0x28).contains(&code) {
        return SLOT_TYPE_LOW[(code - 0x01) as usize].to_string();
    }
    if code == 0x30 {
        return SLOT_TYPE_CXL.to_string();
    }
    if (0xA0..=0xC6).contains(&code) {
        let s = SLOT_TYPE_HIGH[(code - 0xA0) as usize];
        if !s.is_empty() {
            return s.to_string();
        }
    }
    format!("Slot type {code:#x}")
}

// Spec reference: SMBIOS DSP0134 §7.10.2.
const SLOT_BUS_WIDTH: &[&str] = &[
    "Other",       // 0x01
    "Unknown",     // 0x02
    "8 bit",       // 0x03
    "16 bit",      // 0x04
    "32 bit",      // 0x05
    "64 bit",      // 0x06
    "128 bit",     // 0x07
    "1x or x1",    // 0x08
    "2x or x2",    // 0x09
    "4x or x4",    // 0x0A
    "8x or x8",    // 0x0B
    "12x or x12",  // 0x0C
    "16x or x16",  // 0x0D
    "32x or x32",  // 0x0E
];

fn slot_bus_width_name(code: u8) -> String {
    if (0x01..=0x0E).contains(&code) {
        return SLOT_BUS_WIDTH[(code - 0x01) as usize].to_string();
    }
    format!("Bus width {code:#x}")
}

// Spec reference: SMBIOS DSP0134 §7.10.3.
const SLOT_CURRENT_USAGE: &[&str] = &[
    "Other",       // 0x01
    "Unknown",     // 0x02
    "Available",   // 0x03
    "In Use",      // 0x04
    "Unavailable", // 0x05
];

fn slot_current_usage_name(code: u8) -> String {
    if (0x01..=0x05).contains(&code) {
        return SLOT_CURRENT_USAGE[(code - 0x01) as usize].to_string();
    }
    format!("Usage {code:#x}")
}

// Spec reference: SMBIOS DSP0134 §7.10.4.
const SLOT_LENGTH: &[&str] = &[
    "Other",                    // 0x01
    "Unknown",                  // 0x02
    "Short",                    // 0x03
    "Long",                     // 0x04
    "2.5\" drive form factor",  // 0x05
    "3.5\" drive form factor",  // 0x06
];

fn slot_length_name(code: u8) -> String {
    if (0x01..=0x06).contains(&code) {
        return SLOT_LENGTH[(code - 0x01) as usize].to_string();
    }
    format!("Length {code:#x}")
}
