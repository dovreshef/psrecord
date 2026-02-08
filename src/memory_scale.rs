#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryUnit {
    Kilobytes,
    Megabytes,
    Gigabytes,
    Terabytes,
}

impl MemoryUnit {
    pub fn for_peak_bytes(peak_bytes: u64) -> Self {
        const MIB: u64 = 1024 * 1024;
        const GIB: u64 = 1024 * 1024 * 1024;
        const TIB: u64 = 1024 * 1024 * 1024 * 1024;

        if peak_bytes < MIB {
            Self::Kilobytes
        } else if peak_bytes < GIB {
            Self::Megabytes
        } else if peak_bytes < TIB {
            Self::Gigabytes
        } else {
            Self::Terabytes
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Kilobytes => "KB",
            Self::Megabytes => "MB",
            Self::Gigabytes => "GB",
            Self::Terabytes => "TB",
        }
    }

    pub const fn bytes_per_unit(self) -> u64 {
        match self {
            Self::Kilobytes => 1024,
            Self::Megabytes => 1024 * 1024,
            Self::Gigabytes => 1024 * 1024 * 1024,
            Self::Terabytes => 1024 * 1024 * 1024 * 1024,
        }
    }

    pub fn scale_bytes(self, bytes: u64) -> f64 {
        u64_to_f64(bytes) / u64_to_f64(self.bytes_per_unit())
    }
}

fn u64_to_f64(value: u64) -> f64 {
    let upper = u32::try_from(value >> 32).unwrap_or_default();
    let lower = u32::try_from(value & 0xFFFF_FFFF).unwrap_or_default();
    f64::from(upper) * 4_294_967_296.0 + f64::from(lower)
}

#[cfg(test)]
mod tests {
    use super::MemoryUnit;

    #[test]
    fn picks_kb_below_one_megabyte() {
        assert_eq!(
            MemoryUnit::for_peak_bytes(512 * 1024),
            MemoryUnit::Kilobytes
        );
    }

    #[test]
    fn picks_mb_below_one_gigabyte() {
        assert_eq!(
            MemoryUnit::for_peak_bytes(256 * 1024 * 1024),
            MemoryUnit::Megabytes
        );
    }

    #[test]
    fn scales_bytes_to_selected_unit() {
        let scaled_kilobytes = MemoryUnit::Kilobytes.scale_bytes(2048);
        let scaled_megabytes = MemoryUnit::Megabytes.scale_bytes(5 * 1024 * 1024);

        assert!((scaled_kilobytes - 2.0).abs() < f64::EPSILON);
        assert!((scaled_megabytes - 5.0).abs() < f64::EPSILON);
    }
}
