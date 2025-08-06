use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Style, Stylize},
    widgets::{Block, Cell, Padding, Row, Table},
};

use crate::dmi::Release;

#[derive(Debug)]
pub struct Firmware {
    pub vendor: String,
    pub firmware_version: String,
    pub bios_starting_addr_segment: u16,
    pub firmware_release_date: String,
    pub firmware_rom_size: String,
    pub firmware_characteristics: FirmwareCharacteristics,
    pub firmware_characteristics_exentions: FirmwareCharacteristicsExtension,
    pub platform_firmware_release: Release,
    pub embedded_controller_firmware_release: Release,
}

impl From<(Vec<u8>, Vec<String>, u8)> for Firmware {
    fn from((data, text, _length): (Vec<u8>, Vec<String>, u8)) -> Self {
        //TODO: handle different values of length
        let rom_size = {
            let firmware_rom_size: u16 = (data[5] as u16 + 1) * 64;

            if firmware_rom_size < (16 * 1024) {
                format!("{firmware_rom_size}K")
            } else {
                let unit = (data[20] & 0b11000000) >> 6;

                let unit = match unit {
                    0b00 => "M",
                    0b01 => "G",
                    _ => unreachable!(),
                };

                let value = u16::from_le_bytes([(data[20] & 0b00111111), data[21]]);

                format!("{value}{unit}")
            }
        };

        Self {
            vendor: text[data[0].saturating_sub(1) as usize].clone(),
            firmware_version: text[data[1].saturating_sub(1) as usize].clone(),
            bios_starting_addr_segment: u16::from_le_bytes([data[2], data[3]]),
            firmware_release_date: text[data[4].saturating_sub(1) as usize].clone(),
            firmware_rom_size: rom_size,
            firmware_characteristics: FirmwareCharacteristics::from(&data[6..14]),
            firmware_characteristics_exentions: FirmwareCharacteristicsExtension::from([
                data[14], data[15],
            ]),
            platform_firmware_release: Release::new(data[16], data[17]),
            embedded_controller_firmware_release: Release::new(data[18], data[19]),
        }
    }
}

impl Firmware {
    pub fn render(&self, frame: &mut Frame, block: Rect) {
        let (infos_block, characteristics_block) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(15), Constraint::Fill(1)])
                .split(block);

            (chunks[0], chunks[1])
        };

        let rows = vec![
            Row::new(vec![
                Cell::from("Vendor").bold(),
                Cell::from(self.vendor.clone()),
            ]),
            Row::new(vec![
                Cell::from("Firmware Version").bold(),
                Cell::from(self.firmware_version.clone()),
            ]),
            Row::new(vec![
                Cell::from("Firmware Release Date").bold(),
                Cell::from(self.firmware_release_date.clone()),
            ]),
            Row::new(vec![
                Cell::from("Platform Firmware Release").bold(),
                Cell::from(self.platform_firmware_release.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Embedded Controller Firmware Release").bold(),
                Cell::from(self.embedded_controller_firmware_release.to_string()),
            ]),
            Row::new(vec![
                Cell::from("Bios Starting Addr Segment").bold(),
                Cell::from(format!("0x{:X}", self.bios_starting_addr_segment)),
            ]),
            Row::new(vec![
                Cell::from("Firmware ROM size").bold(),
                Cell::from(self.firmware_rom_size.clone()),
            ]),
        ];
        let widths = [Constraint::Length(40), Constraint::Fill(1)];
        let table = Table::new(rows, widths).block(Block::new().padding(Padding::uniform(2)));

        frame.render_widget(table, infos_block.inner(Margin::new(2, 0)));

        // characteristics
        let mut rows = Vec::new();

        if self.firmware_characteristics.supported {
            if self.firmware_characteristics.isa {
                rows.push(Row::new(vec![
                    Cell::from("ISA").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics.mca {
                rows.push(Row::new(vec![
                    Cell::from("MCA").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics.eisa {
                rows.push(Row::new(vec![
                    Cell::from("EISA").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics.pci {
                rows.push(Row::new(vec![
                    Cell::from("PCI").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics.pcmcia {
                rows.push(Row::new(vec![
                    Cell::from("PCMCIA").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics.plug_and_play {
                rows.push(Row::new(vec![
                    Cell::from("Plug and Play").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics.apm {
                rows.push(Row::new(vec![
                    Cell::from("APM").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics.firmware_is_upgradeable {
                rows.push(Row::new(vec![
                    Cell::from("Firmware is upgradeable").bold(),
                    Cell::from("Yes").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics.firmware_shadowing {
                rows.push(Row::new(vec![
                    Cell::from("Firmware Shadowing").bold(),
                    Cell::from("Allowed").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics.vl_vesa {
                rows.push(Row::new(vec![
                    Cell::from("VL-VESA").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics.escd {
                rows.push(Row::new(vec![
                    Cell::from("ESCD").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics.boot_from_cd {
                rows.push(Row::new(vec![
                    Cell::from("Boot from CD").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics.selectable_boot {
                rows.push(Row::new(vec![
                    Cell::from("Selectable boot").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics.firmware_rom_is_socketed {
                rows.push(Row::new(vec![
                    Cell::from("Firmware ROM is socketed").bold(),
                    Cell::from("Yes").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics.boot_from_pcmcia {
                rows.push(Row::new(vec![
                    Cell::from("Boot from PCMCIA").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics.edd_specification {
                rows.push(Row::new(vec![
                    Cell::from("EDD Specification").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self
                .firmware_characteristics
                .int_13h_japanese_floppy_for_nec
            {
                rows.push(Row::new(vec![
                    Cell::from(
                        "Japanese floppy for NEC 9800 1.2 MB (3.5”, 1K bytes/sector, 360 RPM)",
                    )
                    .bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self
                .firmware_characteristics
                .int_13h_japanese_floppy_for_toshiba
            {
                rows.push(Row::new(vec![
                    Cell::from("Japanese floppy for Toshiba 1.2 MB (3.5”, 360 RPM)").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics.int_13h_360 {
                rows.push(Row::new(vec![
                    Cell::from("5.25” / 360 KB floppy services").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics.int_13h_1_2 {
                rows.push(Row::new(vec![
                    Cell::from("5.25” /1.2 MB floppy services").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics.int_13h_720 {
                rows.push(Row::new(vec![
                    Cell::from("3.5” / 720 KB floppy services").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics.int_13h_2_88 {
                rows.push(Row::new(vec![
                    Cell::from("3.5” / 2.88 MB floppy services").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics.int_5h_print_screen {
                rows.push(Row::new(vec![
                    Cell::from("Print screen service").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics.int_9h_8042 {
                rows.push(Row::new(vec![
                    Cell::from("8042 keyboard services").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics.int_14h_serial_service {
                rows.push(Row::new(vec![
                    Cell::from("Serial services").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics.int_17h_printer_service {
                rows.push(Row::new(vec![
                    Cell::from("Printer services").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics.int_10h_cga {
                rows.push(Row::new(vec![
                    Cell::from("CGA/Mono video services").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics.nec_pc_98 {
                rows.push(Row::new(vec![
                    Cell::from("NEC PC-98").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            // Exyended characteristics

            if self.firmware_characteristics_exentions.acpi {
                rows.push(Row::new(vec![
                    Cell::from("ACPI").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics_exentions.usb_legacy {
                rows.push(Row::new(vec![
                    Cell::from("Usb legacy").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics_exentions.agp {
                rows.push(Row::new(vec![
                    Cell::from("AGP").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics_exentions.i2o_boot {
                rows.push(Row::new(vec![
                    Cell::from("I2O").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self
                .firmware_characteristics_exentions
                .ls_120_superdisk_boot
            {
                rows.push(Row::new(vec![
                    Cell::from("LS-120 SuperDisk boot").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics_exentions.atapi_zip_drive_boot {
                rows.push(Row::new(vec![
                    Cell::from("ATAPI ZIP drive boot").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics_exentions._1394_boot {
                rows.push(Row::new(vec![
                    Cell::from("1394 boot").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics_exentions.smart_battery {
                rows.push(Row::new(vec![
                    Cell::from("Smart Battery").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics_exentions.bios_boot_spec {
                rows.push(Row::new(vec![
                    Cell::from("BIOS Boot Specification").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self
                .firmware_characteristics_exentions
                .function_key_initiated_network_service
            {
                rows.push(Row::new(vec![
                    Cell::from("Function key-initiated network service boot").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self
                .firmware_characteristics_exentions
                .enable_targeted_content_distribution
            {
                rows.push(Row::new(vec![
                    Cell::from("Targeted content distribution.").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics_exentions.uefi_spec {
                rows.push(Row::new(vec![
                    Cell::from("UEFI Specification").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self.firmware_characteristics_exentions.virtual_machine {
                rows.push(Row::new(vec![
                    Cell::from("SMBIOS table describes a virtual machine").bold(),
                    Cell::from("Yes").style(Style::new().green()),
                ]));
            }

            if self
                .firmware_characteristics_exentions
                .manufacturing_mode_is_supported
            {
                rows.push(Row::new(vec![
                    Cell::from("Manufacturing mode boot").bold(),
                    Cell::from("Supported").style(Style::new().green()),
                ]));
            }

            if self
                .firmware_characteristics_exentions
                .manufacturing_mode_is_enabled
            {
                rows.push(Row::new(vec![
                    Cell::from("Manufacturing mode enabled").bold(),
                    Cell::from("Yes").style(Style::new().green()),
                ]));
            }

            let widths = [Constraint::Length(40), Constraint::Fill(1)];
            let table = Table::new(rows, widths).block(Block::new().padding(Padding::uniform(2)));

            frame.render_widget(table, characteristics_block.inner(Margin::new(2, 0)));
        }
    }
}

#[derive(Debug)]
pub struct FirmwareCharacteristics {
    pub supported: bool,
    pub isa: bool,
    pub mca: bool,
    pub eisa: bool,
    pub pci: bool,
    pub pcmcia: bool,
    pub plug_and_play: bool,
    pub apm: bool,
    pub firmware_is_upgradeable: bool,
    pub firmware_shadowing: bool,
    pub vl_vesa: bool,
    pub escd: bool,
    pub boot_from_cd: bool,
    pub selectable_boot: bool,
    pub firmware_rom_is_socketed: bool,
    pub boot_from_pcmcia: bool,
    pub edd_specification: bool,
    pub int_13h_japanese_floppy_for_nec: bool,
    pub int_13h_japanese_floppy_for_toshiba: bool,
    pub int_13h_360: bool,
    pub int_13h_1_2: bool,
    pub int_13h_720: bool,
    pub int_13h_2_88: bool,
    pub int_5h_print_screen: bool,
    pub int_9h_8042: bool,
    pub int_14h_serial_service: bool,
    pub int_17h_printer_service: bool,
    pub int_10h_cga: bool,
    pub nec_pc_98: bool,
}

impl From<&[u8]> for FirmwareCharacteristics {
    fn from(value: &[u8]) -> Self {
        let bits = u64::from_le_bytes(value.try_into().unwrap());

        Self {
            supported: bits & (1 << 3) == 0,
            isa: bits & (1 << 4) != 0,
            mca: bits & (1 << 5) != 0,
            eisa: bits & (1 << 6) != 0,
            pci: bits & (1 << 7) != 0,
            pcmcia: bits & (1 << 8) != 0,
            plug_and_play: bits & (1 << 9) != 0,
            apm: bits & (1 << 10) != 0,
            firmware_is_upgradeable: bits & (1 << 11) != 0,
            firmware_shadowing: bits & (1 << 12) != 0,
            vl_vesa: bits & (1 << 13) != 0,
            escd: bits & (1 << 14) != 0,
            boot_from_cd: bits & (1 << 15) != 0,
            selectable_boot: bits & (1 << 16) != 0,
            firmware_rom_is_socketed: bits & (1 << 17) != 0,
            boot_from_pcmcia: bits & (1 << 18) != 0,
            edd_specification: bits & (1 << 19) != 0,
            int_13h_japanese_floppy_for_nec: bits & (1 << 20) != 0,
            int_13h_japanese_floppy_for_toshiba: bits & (1 << 21) != 0,
            int_13h_360: bits & (1 << 22) != 0,
            int_13h_1_2: bits & (1 << 23) != 0,
            int_13h_720: bits & (1 << 24) != 0,
            int_13h_2_88: bits & (1 << 25) != 0,
            int_5h_print_screen: bits & (1 << 26) != 0,
            int_9h_8042: bits & (1 << 27) != 0,
            int_14h_serial_service: bits & (1 << 28) != 0,
            int_17h_printer_service: bits & (1 << 29) != 0,
            int_10h_cga: bits & (1 << 30) != 0,
            nec_pc_98: bits & (1 << 31) != 0,
        }
    }
}

#[derive(Debug)]
pub struct FirmwareCharacteristicsExtension {
    pub acpi: bool,
    pub usb_legacy: bool,
    pub agp: bool,
    pub i2o_boot: bool,
    pub ls_120_superdisk_boot: bool,
    pub atapi_zip_drive_boot: bool,
    pub _1394_boot: bool,
    pub smart_battery: bool,
    pub bios_boot_spec: bool,
    pub function_key_initiated_network_service: bool,
    pub enable_targeted_content_distribution: bool,
    pub uefi_spec: bool,
    pub virtual_machine: bool,
    pub manufacturing_mode_is_supported: bool,
    pub manufacturing_mode_is_enabled: bool,
}

impl From<[u8; 2]> for FirmwareCharacteristicsExtension {
    fn from(value: [u8; 2]) -> Self {
        Self {
            acpi: value[0] & 1 != 0,
            usb_legacy: value[0] & (1 << 1) != 0,
            agp: value[0] & (1 << 2) != 0,
            i2o_boot: value[0] & (1 << 3) != 0,
            ls_120_superdisk_boot: value[0] & (1 << 4) != 0,
            atapi_zip_drive_boot: value[0] & (1 << 5) != 0,
            _1394_boot: value[0] & (1 << 6) != 0,
            smart_battery: value[0] & (1 << 7) != 0,
            bios_boot_spec: value[1] & 1 != 0,
            function_key_initiated_network_service: value[1] & (1 << 1) != 0,
            enable_targeted_content_distribution: value[1] & (1 << 2) != 0,
            uefi_spec: value[1] & (1 << 3) != 0,
            virtual_machine: value[1] & (1 << 4) != 0,
            manufacturing_mode_is_supported: value[1] & (1 << 5) != 0,
            manufacturing_mode_is_enabled: value[1] & (1 << 6) != 0,
        }
    }
}
