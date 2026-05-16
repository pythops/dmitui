// SMBIOS Type 7 (Cache Information). Spec reference: DSP0134 §7.8.

#[derive(Debug)]
pub struct Cache {
    pub handle: u16,
    installed_size: CacheSize,
    cache_type: CacheType,
}

impl Cache {
    pub fn parse(handle: u16, data: Vec<u8>) -> Self {
        let installed_size_field = u16::from_le_bytes(data[5..7].try_into().unwrap());
        let installed_size_2 = if data.len() >= 23 {
            Some(u32::from_le_bytes(data[19..23].try_into().unwrap()))
        } else {
            None
        };
        let installed_size = CacheSize::from_fields(installed_size_field, installed_size_2);

        let cache_type = data
            .get(13)
            .copied()
            .map_or(CacheType::Unknown, CacheType::from);

        Self {
            handle,
            installed_size,
            cache_type,
        }
    }

    pub fn summary(&self) -> String {
        if matches!(self.installed_size, CacheSize::NotInstalled) {
            return "Not installed".to_string();
        }
        format!("{}, {}", self.installed_size, self.cache_type)
    }
}

#[derive(Debug)]
enum CacheSize {
    NotInstalled,
    Unknown,
    Kilobytes(u64),
}

impl CacheSize {
    fn from_fields(size_field: u16, size2_field: Option<u32>) -> Self {
        match size_field {
            0 => CacheSize::NotInstalled,
            0xFFFF => match size2_field {
                Some(0) => CacheSize::NotInstalled,
                Some(v) => decode_size_2(v),
                None => CacheSize::Unknown,
            },
            v => decode_size(v),
        }
    }
}

// 16-bit size field: bit 15 = granularity (0 → 1 KB, 1 → 64 KB), bits 0..14 = count.
fn decode_size(v: u16) -> CacheSize {
    let count = (v & 0x7FFF) as u64;
    let granularity_kb: u64 = if v & 0x8000 == 0 { 1 } else { 64 };
    CacheSize::Kilobytes(count * granularity_kb)
}

// 32-bit size field: bit 31 = granularity (0 → 1 KB, 1 → 64 KB), bits 0..30 = count.
fn decode_size_2(v: u32) -> CacheSize {
    let count = (v & 0x7FFF_FFFF) as u64;
    let granularity_kb: u64 = if v & 0x8000_0000 == 0 { 1 } else { 64 };
    CacheSize::Kilobytes(count * granularity_kb)
}

impl std::fmt::Display for CacheSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheSize::NotInstalled => write!(f, "Not installed"),
            CacheSize::Unknown => write!(f, "Unknown"),
            CacheSize::Kilobytes(kb) => {
                if *kb >= 1024 && kb.is_multiple_of(1024) {
                    write!(f, "{} MB", kb / 1024)
                } else if *kb >= 1024 {
                    write!(f, "{:.1} MB", *kb as f64 / 1024.0)
                } else {
                    write!(f, "{kb} KB")
                }
            }
        }
    }
}

#[derive(Debug, strum::Display)]
enum CacheType {
    #[strum(to_string = "Other")]
    Other,
    #[strum(to_string = "Unknown")]
    Unknown,
    #[strum(to_string = "Instruction")]
    Instruction,
    #[strum(to_string = "Data")]
    Data,
    #[strum(to_string = "Unified")]
    Unified,
}

impl From<u8> for CacheType {
    fn from(value: u8) -> Self {
        match value {
            3 => Self::Instruction,
            4 => Self::Data,
            5 => Self::Unified,
            1 => Self::Other,
            _ => Self::Unknown,
        }
    }
}
