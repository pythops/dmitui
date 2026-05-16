mod baseboard;
mod battery;
mod chassis;
mod firmware;
mod memory;
mod system;

use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
};

use anyhow::{Result, bail};

use crate::dmi::baseboard::Baseboard;
use crate::dmi::battery::Battery;
use crate::dmi::chassis::Chassis;
use crate::dmi::firmware::Firmware;
use crate::dmi::memory::{Memory, PhysicalMemoryArray};
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
    firmware: Option<Firmware>,
    system: Option<System>,
    baseboard: Option<Baseboard>,
    chassis: Option<Chassis>,
    memory: Option<Memory>,
    battery: Option<Battery>,
    pub focused_section: FocusedSection,
}

#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FocusedSection {
    Firmware,
    System,
    Baseboard,
    Chassis,
    Memory,
    Battery,
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
            16 => StructureType::PhysicalMemoryArray,
            22 => StructureType::Battery,
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
    PhysicalMemoryArray = 16,
    Battery = 22,
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
        let mut memory: Option<Memory> = None;
        let mut battery: Option<Battery> = None;

        let dmi_file_path = Path::new("/sys/firmware/dmi/tables/DMI");

        match dmi_file_path.try_exists() {
            Ok(true) => {}
            Ok(false) | Err(_) => bail!("No SMBIOS found"),
        }

        let mem_file = File::open(dmi_file_path)?;
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

            // Read strings. The string-set ends with an extra NUL after the
            // last string's terminator, so for a structure with no strings the
            // formatted area is followed by two NUL bytes.
            let mut text: Vec<String> = Vec::new();
            let mut saw_leading_zero = false;

            loop {
                let mut string_buf = Vec::new();
                match file.read_until(0, &mut string_buf)? {
                    0 => break,
                    1 => {
                        // Empty entry (just the terminator byte).
                        if !text.is_empty() || saw_leading_zero {
                            break;
                        }
                        saw_leading_zero = true;
                    }
                    _ => {
                        string_buf.pop();
                        text.push(String::from_utf8_lossy(&string_buf).to_string());
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
                StructureType::PhysicalMemoryArray => {
                    memory = Some(Memory {
                        physical_memory_array: PhysicalMemoryArray::from(data.as_slice()),
                    });
                }
                StructureType::Battery => {
                    battery = Some(Battery::from((data, text)));
                }
                _ => {}
            }
        }

        let focused_section = [
            (FocusedSection::Firmware, firmware.is_some()),
            (FocusedSection::System, system.is_some()),
            (FocusedSection::Baseboard, baseboard.is_some()),
            (FocusedSection::Chassis, chassis.is_some()),
            (FocusedSection::Memory, memory.is_some()),
            (FocusedSection::Battery, battery.is_some()),
        ]
        .into_iter()
        .find_map(|(s, present)| present.then_some(s))
        .ok_or_else(|| anyhow::anyhow!("No supported DMI structures found"))?;

        Ok(Self {
            firmware,
            system,
            baseboard,
            chassis,
            memory,
            battery,
            focused_section,
        })
    }

    fn available_sections(&self) -> Vec<FocusedSection> {
        let mut sections = Vec::with_capacity(6);
        if self.firmware.is_some() {
            sections.push(FocusedSection::Firmware);
        }
        if self.system.is_some() {
            sections.push(FocusedSection::System);
        }
        if self.baseboard.is_some() {
            sections.push(FocusedSection::Baseboard);
        }
        if self.chassis.is_some() {
            sections.push(FocusedSection::Chassis);
        }
        if self.memory.is_some() {
            sections.push(FocusedSection::Memory);
        }
        if self.battery.is_some() {
            sections.push(FocusedSection::Battery);
        }
        sections
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) {
        let sections = self.available_sections();
        let Some(idx) = sections.iter().position(|s| *s == self.focused_section) else {
            return;
        };

        match key_event.code {
            KeyCode::Tab => {
                self.focused_section = sections[(idx + 1) % sections.len()];
            }
            KeyCode::BackTab => {
                self.focused_section = sections[(idx + sections.len() - 1) % sections.len()];
            }
            _ => {}
        }
    }

    fn title_span(&self, header_section: FocusedSection) -> Span<'_> {
        let label = match header_section {
            FocusedSection::Firmware => "  Firmware  ",
            FocusedSection::System => "  System  ",
            FocusedSection::Baseboard => "  Baseboard  ",
            FocusedSection::Chassis => "  Chassis  ",
            FocusedSection::Memory => "  Memory  ",
            FocusedSection::Battery => "  Battery  ",
        };

        if self.focused_section == header_section {
            Span::styled(
                label,
                Style::default().bg(Color::Yellow).fg(Color::Black).bold(),
            )
        } else {
            Span::from(label).fg(Color::DarkGray)
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

        let title_spans: Vec<Span<'_>> = self
            .available_sections()
            .into_iter()
            .map(|s| self.title_span(s))
            .collect();

        frame.render_widget(
            Block::default()
                .title(Line::from(title_spans))
                .title_alignment(Alignment::Left)
                .padding(Padding::top(1))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default())
                .border_style(Style::default().fg(Color::Yellow)),
            section_block,
        );

        // Help banner
        let message = Line::from("⇆ : Navigation").centered().cyan();

        frame.render_widget(message, help_block);

        match self.focused_section {
            FocusedSection::Firmware => {
                if let Some(firmware) = &self.firmware {
                    firmware.render(frame, section_block);
                }
            }
            FocusedSection::System => {
                if let Some(system) = &self.system {
                    system.render(frame, section_block);
                }
            }
            FocusedSection::Baseboard => {
                if let Some(baseboard) = &self.baseboard {
                    baseboard.render(frame, section_block);
                }
            }
            FocusedSection::Chassis => {
                if let Some(chassis) = &self.chassis {
                    chassis.render(frame, section_block);
                }
            }
            FocusedSection::Memory => {
                if let Some(memory) = &mut self.memory {
                    memory.render(frame, section_block);
                }
            }
            FocusedSection::Battery => {
                if let Some(battery) = &self.battery {
                    battery.render(frame, section_block);
                }
            }
        }
    }
}
