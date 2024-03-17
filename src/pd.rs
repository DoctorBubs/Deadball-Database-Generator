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

// To reflect Die notation the PD enum is organized via the following pattern: D[i] is equivalant to d[i], while DM[i] is equivilant to -d[i]
pub enum PD {
    #[serde(rename="d20")]
    D20,
    #[serde(rename="d12")]
    D12,
    #[serde(rename="d8")]
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
