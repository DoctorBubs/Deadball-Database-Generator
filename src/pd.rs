use inquire::Confirm;
use inquire::Select;
use rusqlite::Connection;

use crate::era::Era;
use crate::tier::Tier;

use crate::update_player_db::update_player_db_option;
use crate::update_player_db::UpdatePlayerDb;
use crate::Deserialize;

use crate::Serialize;
use core::fmt;

struct PDInfo(i32, bool);

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
/*  In Deadball, the biggest difference between pitchers is their Pitch Die, which when playing the game corresponds to which die is rolled when the pitcher is used during a game,
as well as if the number generated is positive or negative.

For example,when using a pitcher with a PD of D12, a 12 sided die is rolled, and the number generated is positive.
Inversely, a pitcher with a PD of -D4 rolls a 4 sided die, the the number generated is negative.
*/
// Via Serde, the pitch die is serialized via traditional dice notation.
/// The pitch die enum.
#[derive(Default)]
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
    #[default]
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
    #[serde(rename = "-d20")]
    DM20,
    Custom(i32),
}

impl PD {
    // Returns the max value possible for a pitch die.
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
            //
            Self::Custom(value) => value,
        }
    }

    pub fn get_tier(&self) -> Tier {
        match self {
            Self::D20 => Tier::S(0),
            Self::D12 => Tier::A(0),
            Self::D8 => Tier::B(0),
            Self::D6 => Tier::B(-1),
            Self::D4 => Tier::C(0),
            Self::D0 => Tier::C(-1),
            Self::DM4 => Tier::D(0),
            Self::DM6 => Tier::F(1),
            Self::DM8 => Tier::F(0),
            Self::DM12 => Tier::F(-1),
            Self::DM20 => Tier::F(-2),
            Self::Custom(_) => Tier::C(0),
        }
    }

    fn get_info(&self) -> PDInfo {
        let num = self.to_int();
        let is_positive = num > 0;
        PDInfo(num, is_positive)
    }
    /// Creates a new custom pitch die from an integer.
    pub fn new_custom_pd(num: i32) -> Self {
        Self::Custom(num)
    }
    /// Creates a range of all possible values that can be created by a pitch die.
    fn get_range(&self) -> Vec<i32> {
        let max = self.to_int();
        match max < 0 {
            true => (max..0).collect(),
            false => (1..=max).collect(),
        }
    }
    // Calculates the average result of a roll of the pitch die.
    pub fn get_average(&self) -> f32 {
        let range = self.get_range();
        let range_len = range.len() as f32;
        let range_sum: i32 = range.iter().sum();
        range_sum as f32 / range_len
    }

    pub fn fix_db(
        input_opt: Result<Option<Self>, serde_json::Error>,
        conn: &mut Connection,
        player_id: i64,
        player_name: &str,
        era: Era,
    ) -> Result<Option<Self>, serde_json::Error> {
        match input_opt {
            Ok(value) => Ok(value),
            Err(message) => {
                let message_string = message.to_string();
                let default_error = Err(message);
                println!("There was an error in the pitch dice column for {}, player ID {}. The error was {}.",player_name,player_id ,message_string);
                let user_confirm = Confirm::new(
                    "Would you like to fix this error now? Otherwise, the process will abort",
                )
                .prompt()
                .unwrap_or(false);
                if !user_confirm {
                    return default_error;
                };
                let user_choice =
                    match Confirm::new("Would you like to assign a pitch die to this player?")
                        .prompt()
                    {
                        Err(_) => return default_error,
                        Ok(value) => match value {
                            false => None,
                            true => {
                                let pd_options = era.get_all_pd();
                                match Select::new(
                                    "Please select the pitch die you would like to assign.",
                                    pd_options,
                                )
                                .prompt()
                                {
                                    Err(_) => return default_error,
                                    Ok(value) => Some(value),
                                }
                            }
                        },
                    };

                let update_attempt = update_player_db_option(user_choice, conn, player_id);
                if update_attempt.is_err() {
                    default_error
                } else {
                    Ok(user_choice)
                }
            }
        }
    }
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

impl PartialOrd for PD {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let other_int = other.to_int();
        Some(self.to_int().cmp(&other_int))
    }
}

impl UpdatePlayerDb for PD {
    fn get_column_name(&self) -> &str {
        "PD"
    }
}
