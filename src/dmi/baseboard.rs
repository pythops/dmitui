use std::fmt::Display;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Style, Stylize},
    widgets::{Block, Cell, List, Padding, Row, Table},
};

#[derive(Debug)]
pub struct Baseboard {
    manufacturer: String,
    product: String,
    version: String,
    serial_number: String,
    asset_tag: String,
    features: Vec<Feaures>,
    loacation_in_chassis: String,
    board_type: BoardType,
}

#[derive(Debug)]
enum Feaures {
    HotSwappable,
    Replaceable,
    Removable,
    RequiresOneDaughterBoard,
    HostingBoard,
}

impl Display for Feaures {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HotSwappable => write!(f, "The board is hot swappable"),
            Self::Replaceable => write!(f, "The board is replaceable"),
            Self::Removable => write!(f, "The board is removable"),
            Self::RequiresOneDaughterBoard => write!(
                f,
                "The board requires at least one daughter board or auxiliary card to function properly"
            ),
            Self::HostingBoard => write!(
                f,
                "The board is a hosting board (for example, a motherboard)."
            ),
        }
    }
}

#[derive(Debug)]
enum BoardType {
    Unknown,
    Other,
    ServerBlade,
    ConnectivitySwitch,
    SystemManagementModule,
    ProcessorModule,
    IOModule,
    MemoryModule,
    DaughterBoard,
    MotherBoard,
    ProcessorMemoryModule,
    ProcessorIOModule,
    InterconnectedModule,
}
impl Display for BoardType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => write!(f, "Unknown"),
            Self::Other => write!(f, "Other"),
            Self::ServerBlade => write!(f, "Server Blade"),
            Self::ConnectivitySwitch => write!(f, "Connectivity Switch"),
            Self::SystemManagementModule => write!(f, "System Management Module"),
            Self::ProcessorModule => write!(f, "Processor Module"),
            Self::IOModule => write!(f, "I/O Module"),
            Self::MemoryModule => write!(f, "Memory Module"),
            Self::DaughterBoard => write!(f, "Daughter board"),
            Self::MotherBoard => write!(f, "Motherboard (includes processor, memory, and I/O)"),
            Self::ProcessorMemoryModule => write!(f, "Processor/Memory Module"),
            Self::ProcessorIOModule => write!(f, "Processor/IO Module"),
            Self::InterconnectedModule => write!(f, "Interconnect board"),
        }
    }
}

impl From<u8> for BoardType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Unknown,
            2 => Self::Other,
            3 => Self::ServerBlade,
            4 => Self::ConnectivitySwitch,
            5 => Self::SystemManagementModule,
            6 => Self::ProcessorModule,
            7 => Self::IOModule,
            8 => Self::MemoryModule,
            9 => Self::DaughterBoard,
            10 => Self::MotherBoard,
            11 => Self::ProcessorMemoryModule,
            12 => Self::ProcessorIOModule,
            13 => Self::InterconnectedModule,
            _ => unreachable!(),
        }
    }
}

impl From<(Vec<u8>, Vec<String>)> for Baseboard {
    fn from((data, text): (Vec<u8>, Vec<String>)) -> Self {
        let mut features = Vec::new();

        if data[5] & 1 != 0 {
            features.push(Feaures::HostingBoard);
        };

        if data[5] & (1 << 1) != 0 {
            features.push(Feaures::RequiresOneDaughterBoard);
        };

        if data[5] & (1 << 2) != 0 {
            features.push(Feaures::Removable);
        }

        if data[5] & (1 << 3) != 0 {
            features.push(Feaures::Replaceable);
        }

        if data[5] & (1 << 4) != 0 {
            features.push(Feaures::HotSwappable);
        }

        Self {
            manufacturer: text[data[0].saturating_sub(1) as usize].clone(),
            product: text[data[1].saturating_sub(1) as usize].clone(),
            version: text[data[2].saturating_sub(1) as usize].clone(),
            serial_number: text[data[3].saturating_sub(1) as usize].clone(),
            asset_tag: text[data[4].saturating_sub(1) as usize].clone(),
            features,
            loacation_in_chassis: text[data[8].saturating_sub(1) as usize].clone(),
            board_type: BoardType::from(data[9]),
        }
    }
}

impl Baseboard {
    pub fn render(&self, frame: &mut Frame, block: Rect) {
        let (infos_block, feaures_block) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(15), Constraint::Fill(1)])
                .split(block);

            (chunks[0], chunks[1])
        };

        let rows = vec![
            Row::new(vec![
                Cell::from("Manufacturer").bold(),
                Cell::from(self.manufacturer.clone()),
            ]),
            Row::new(vec![
                Cell::from("Product").bold(),
                Cell::from(self.product.clone()),
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
                Cell::from("Asset Tag").bold(),
                Cell::from(self.asset_tag.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Location in Chassis").bold(),
                Cell::from(self.loacation_in_chassis.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Board Type").bold(),
                Cell::from(self.board_type.to_string()),
            ]),
        ];

        let widths = [Constraint::Length(20), Constraint::Fill(1)];
        let table = Table::new(rows, widths).block(Block::new().padding(Padding::uniform(2)));
        frame.render_widget(table, infos_block.inner(Margin::new(2, 0)));

        let feaures = self.features.iter().map(|feature| format!("* {feature}"));
        let list = List::new(feaures).block(
            Block::new()
                .title("  Features")
                .title_style(Style::new().bold())
                .padding(Padding::symmetric(2, 1)),
        );
        frame.render_widget(list, feaures_block.inner(Margin::new(2, 0)));
    }
}
