use crate::Deserialize;

use crate::Serialize;

struct PDInfo(i32, bool);

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum PD {
    D20,
    D12,
    D8,
    D6,
    D4,
    D0,
    DM4,
    DM6,
    DM8,
    DM12,
    DM20,
}

impl PD {
    pub fn to_int(&self) -> i32 {
        match self {
            Self::D20 => 20,
            Self::D12 => 12,
            Self::D8 => 8,
            Self::D6 => 6,
            Self::D4 => 4,
            Self::D0 => 0,
            Self::DM4 => -4,
            Self::DM6 => -6,
            Self::DM8 => -8,
            Self::DM12 => -12,
            Self::DM20 => -20,
        }
    }

    fn get_info(&self) -> PDInfo {
        let num = self.to_int();
        let is_positive = num > 0;
        PDInfo(num, is_positive)
    }
    pub fn to_string(&self) -> String {
        let PDInfo(num, is_positive) = self.get_info();
        let num_string = num.abs().to_string();
        match is_positive {
            true => format!("d{}", num_string),
            false => format!("-d{}", num_string),
        }
    }
}
