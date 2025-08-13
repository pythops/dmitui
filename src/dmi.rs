mod baseboard;
mod chassis;
mod firmware;
mod system;

use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
};

use anyhow::{Result, bail};

use crate::dmi::baseboard::Baseboard;
use crate::dmi::chassis::Chassis;
use crate::dmi::firmware::Firmware;
use crate::dmi::system::System;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding},
};

#[derive(Debug)]
pub struct DMI {
    firmware: Firmware,
    system: System,
    baseboard: Baseboard,
    chassis: Chassis,
    pub focused_section: FocusedSection,
}

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum FocusedSection {
    Firmware,
    System,
    Baseboard,
    Chassis,
}

#[derive(Debug)]
pub struct Header {
    pub structure_type: StructureType,
    pub length: u8,
    pub handle: u16,
}

impl From<[u8; 4]> for Header {
    fn from(value: [u8; 4]) -> Self {
        let structure_type = match value[0] {
            0 => StructureType::Firmware,
            1 => StructureType::System,
            2 => StructureType::Baseboard,
            3 => StructureType::Chassis,
            13 => StructureType::FirmwareLanguage,
            127 => StructureType::End,
            _ => StructureType::Other,
        };

        Self {
            structure_type,
            length: value[1],
            handle: u16::from_be_bytes([value[2], value[3]]),
        }
    }
}

#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum StructureType {
    Firmware = 0,
    System = 1,
    Baseboard = 2,
    Chassis = 3,
    FirmwareLanguage = 13,
    End = 127,
    Other = 255,
}

// https://www.dmtf.org/dsp/DSP0134
// https://www.dmtf.org/sites/default/files/standards/documents/DSP0134_3.8.0.pdf
impl DMI {
    pub fn new() -> Result<Self> {
        let mut firmware: Option<Firmware> = None;
        let mut system: Option<System> = None;
        let mut baseboard: Option<Baseboard> = None;
        let mut chassis: Option<Chassis> = None;

        let dmi_file_path = Path::new("/sys/firmware/dmi/tables/DMI");

        match dmi_file_path.try_exists() {
            Ok(true) => {}
            Ok(false) | Err(_) => {
                eprintln!("No SMBIOS found");
                std::process::exit(1);
            }
        }

        let mem_file = File::open("/sys/firmware/dmi/tables/DMI")?;
        let mut file = BufReader::new(mem_file);

        loop {
            // Read header
            let mut header_buffer: [u8; 4] = [0; 4];
            file.read_exact(&mut header_buffer)?;
            let header = Header::from(header_buffer);
            if header.structure_type == StructureType::End {
                break;
            }

            if header.length < 4 {
                bail!("Header size < 4");
            }

            // Read structure
            let mut data = vec![0; header.length.saturating_sub(4) as usize];
            file.read_exact(&mut data)?;

            // Read Strings
            let mut text: Vec<String> = Vec::new();

            let mut previous_read_zero: bool = false;
            let mut previous_read_string: bool = false;

            loop {
                let mut string_buf = Vec::new();
                if let Ok(number_of_bytes_read) = file.read_until(0, &mut string_buf) {
                    if number_of_bytes_read == 1 {
                        if previous_read_zero {
                            break;
                        } else {
                            if previous_read_string {
                                break;
                            }
                            previous_read_zero = true;
                        }
                    } else {
                        string_buf.pop();
                        text.push(String::from_utf8_lossy(&string_buf).to_string());
                        previous_read_string = true;
                    }
                }
            }

            match header.structure_type {
                StructureType::Firmware => {
                    firmware = Some(Firmware::from((data, text, header.length)));
                }
                StructureType::System => {
                    system = Some(System::from((data, text)));
                }
                StructureType::Baseboard => {
                    baseboard = Some(Baseboard::from((data, text)));
                }
                StructureType::Chassis => {
                    chassis = Some(Chassis::from((data, text)));
                }
                StructureType::FirmwareLanguage => {
                    let language_infos = firmware::LanguageInfos::from((data, text));

                    if let Some(firmware) = &mut firmware {
                        firmware.language_infos = Some(language_infos);
                    }
                }
                _ => {}
            }
        }

        Ok(Self {
            firmware: firmware.unwrap(),
            system: system.unwrap(),
            baseboard: baseboard.unwrap(),
            chassis: chassis.unwrap(),
            focused_section: FocusedSection::Firmware,
        })
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Up | KeyCode::Char('k') => {}
            KeyCode::Down | KeyCode::Char('j') => {}
            KeyCode::Tab => match self.focused_section {
                FocusedSection::Firmware => self.focused_section = FocusedSection::System,
                FocusedSection::System => self.focused_section = FocusedSection::Baseboard,
                FocusedSection::Baseboard => self.focused_section = FocusedSection::Chassis,
                FocusedSection::Chassis => self.focused_section = FocusedSection::Firmware,
            },
            KeyCode::BackTab => match self.focused_section {
                FocusedSection::Firmware => self.focused_section = FocusedSection::Chassis,
                FocusedSection::System => self.focused_section = FocusedSection::Firmware,
                FocusedSection::Baseboard => self.focused_section = FocusedSection::System,
                FocusedSection::Chassis => self.focused_section = FocusedSection::Baseboard,
            },
            _ => {}
        }
    }

    fn title_span(&self, header_section: FocusedSection) -> Span<'_> {
        let is_focused = self.focused_section == header_section;
        match header_section {
            FocusedSection::Firmware => {
                if is_focused {
                    Span::styled(
                        "  Firmware  ",
                        Style::default().bg(Color::Yellow).fg(Color::White).bold(),
                    )
                } else {
                    Span::from("  Firmware  ").fg(Color::DarkGray)
                }
            }
            FocusedSection::System => {
                if is_focused {
                    Span::styled(
                        "  System  ",
                        Style::default().bg(Color::Yellow).fg(Color::White).bold(),
                    )
                } else {
                    Span::from("  System  ").fg(Color::DarkGray)
                }
            }
            FocusedSection::Baseboard => {
                if is_focused {
                    Span::styled(
                        "  Baseboard  ",
                        Style::default().bg(Color::Yellow).fg(Color::White).bold(),
                    )
                } else {
                    Span::from("  Baseboard  ").fg(Color::DarkGray)
                }
            }
            FocusedSection::Chassis => {
                if is_focused {
                    Span::styled(
                        "  Chassis  ",
                        Style::default().bg(Color::Yellow).fg(Color::White).bold(),
                    )
                } else {
                    Span::from("  Chassis  ").fg(Color::DarkGray)
                }
            }
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let (section_block, help_block) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Fill(1), Constraint::Length(3)])
                .flex(ratatui::layout::Flex::SpaceBetween)
                .split(frame.area());

            (chunks[0], chunks[1])
        };

        frame.render_widget(
            Block::default()
                .title(Line::from(vec![
                    self.title_span(FocusedSection::Firmware),
                    self.title_span(FocusedSection::System),
                    self.title_span(FocusedSection::Baseboard),
                    self.title_span(FocusedSection::Chassis),
                ]))
                .title_alignment(Alignment::Left)
                .padding(Padding::top(1))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default())
                .border_style(Style::default().fg(Color::Yellow)),
            section_block,
        );

        // Help banner
        let message = Line::from("↑,k : Up | ↓,j : Down | ⇆ : Navigation")
            .centered()
            .cyan();

        frame.render_widget(message, help_block);

        match self.focused_section {
            FocusedSection::Firmware => {
                self.firmware.render(frame, section_block);
            }
            FocusedSection::System => {
                self.system.render(frame, section_block);
            }
            FocusedSection::Baseboard => {
                self.baseboard.render(frame, section_block);
            }
            FocusedSection::Chassis => {
                self.chassis.render(frame, section_block);
            }
        }
    }
}
