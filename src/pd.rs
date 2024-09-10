use crate::Deserialize;

use crate::Serialize;
use core::fmt;

struct PDInfo(i32, bool);

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]

/*  In Deadball, the bigggest difference between pitchers is their Pitch Die, which when playing the game corresponds to which die is rolled when the pitcher is used during a game,
as well as if the numger generated is postiive or negative.

For example,when using a pitcher with a PD of D12, a 12 sided die is rolled, and the number generated is positive.
Inversely, a pitcher with a PD of -D4 rolls a 4 sided die, the the number generated is negative.
*/





// Via Serde, the pitch die is serialized via traditional dice notation
pub enum PD {
    #[serde(rename = "d20")]
    D20,
    #[serde(rename = "d12")]
    D12,
    #[serde(rename = "d8")]
    D8,
    #[serde(rename = "d6")]
    D6,
    #[serde(rename = "d4")]
    D4,
    #[serde(rename = "No dice")]
    D0,
    #[serde(rename = "-d4")]
    DM4,
    #[serde(rename = "-d6")]
    DM6,
    #[serde(rename = "-d8")]
    DM8,
    #[serde(rename = "-d12")]
    DM12,
    #[serde(rename = "d20")]
    DM20,
}

impl PD {
    pub fn to_int(self) -> i32 {
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
    /*pub fn to_string(self) -> String {
        let PDInfo(num, is_positive) = self.get_info();
        let num_string = num.abs().to_string();
        match is_positive {
            true => format!("d{}", num_string),
            false => format!("-d{}", num_string),
        }
    }*/
}

impl fmt::Display for PD {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let PDInfo(num, is_positive) = self.get_info();
        let num_string = num.abs().to_string();
        let chars = match is_positive {
            true => format!("d{}", num_string),
            false => format!("-d{}", num_string),
        };

        write!(f, "{}", chars)
    }
}
