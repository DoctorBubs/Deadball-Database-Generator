use core::fmt;
use inquire::{Confirm, Select};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::{era::Era, pd::PD};

/// If a player is a position player, we lists what type of pitcher they are, and what position they can field.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct TwoWayInfo {
    pitcher_type: PlayerPosition,
    fielder_type: PlayerPosition,
}

impl fmt::Display for TwoWayInfo {
    /// Formats a TwoWayInfo to be the pitcher followed by a slash and the fielder type.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pitcher_string = serde_json::to_string(&self.pitcher_type).unwrap();
        let fielder_string = serde_json::to_string(&self.fielder_type).unwrap();
        let chars = format!("{}/{}", pitcher_string, fielder_string);
        write!(f, "{}", chars)
    }
}

impl TwoWayInfo {
    pub fn is_valid(&self) -> Result<&TwoWayInfo, String> {
        let p_category = PositionCategory::Pitcher;
        let error_str = match (
            self.pitcher_type.matches_cat(p_category),
            self.fielder_type.matches_cat(p_category),
        ) {
            (true, false) => match self {
                Self {
                    pitcher_type: PlayerPosition::TwoWay(_),
                    ..
                }
                | Self {
                    fielder_type: PlayerPosition::TwoWay(_),
                    ..
                } => Some(
                    "A two way player can not have its pitcher or fielder type be a two way player",
                ),
                _ => None,
            },
            _ => Some("A two way player must have a valid pitcher and fielder type"),
        };
        match error_str {
            Some(error_str) => {
                let error_message = format!("{}\n{}", error_str, self);
                Err(error_message)
            }
            None => Ok(self),
        }
    }

    pub fn get_pitcher_type(&self) -> Result<&PlayerPosition, String> {
        let self_ref = self.is_valid()?;
        Ok(&self_ref.pitcher_type)
    }

    pub fn new(pitcher_type: PlayerPosition, fielder_type: PlayerPosition) -> Self {
        TwoWayInfo {
            pitcher_type,
            fielder_type,
        }
    }
}

//. Represents what type of position a player holds.
#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum PositionCategory {
    Pitcher,
    Infielder,
    Outfielder,
    Catcher,
}

/// Represents the position that is assigned to a player.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Default)]
pub enum PlayerPosition {
    P,
    SP,
    RP,
    C,
    #[serde(rename = "1B")]
    FirstBase,
    #[serde(rename = "2B")]
    SecondBase,
    #[serde(rename = "3B")]
    ThirdBase,
    SS,
    LF,
    CF,
    RF,
    OF,
    INF,
    #[default]
    UT,
    TwoWay(Box<TwoWayInfo>),
}

impl fmt::Display for PlayerPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // We check if this is a two way player. If it is, we format it's value
        if let Self::TwoWay(info) = self {
            info.fmt(f)
        } else {
            let chars = serde_json::to_string(&self).unwrap();
            write!(f, "{}", chars)
        }
    }
}

impl PlayerPosition {
    pub fn get_pitcher_type(&self) -> Result<&PlayerPosition, String> {
        match self {
            Self::TwoWay(two_way_info) => two_way_info.get_pitcher_type(),
            _ => match self.matches_cat(PositionCategory::Pitcher) {
                false => Err("Not a pitcher".to_string()),
                true => Ok(self),
            },
        }
    }

    pub fn matches_cat(&self, category: PositionCategory) -> bool {
        if let Self::TwoWay(two_way_info) = self {
            match category {
                PositionCategory::Pitcher => true,
                _ => two_way_info.fielder_type.matches_cat(category),
            }
        } else {
            match category {
                PositionCategory::Outfielder => {
                    matches!(self, Self::LF | Self::CF | Self::RF | Self::OF | Self::UT)
                }
                PositionCategory::Catcher => self == &Self::C,
                PositionCategory::Infielder => matches!(
                    self,
                    Self::FirstBase
                        | Self::SecondBase
                        | Self::ThirdBase
                        | Self::SS
                        | Self::INF
                        | Self::UT,
                ),

                PositionCategory::Pitcher => matches!(self, Self::P | Self::SP | Self::RP),
            }
        }
    }
    pub fn get_all_batter_positions() -> Vec<Self> {
        vec![
            PlayerPosition::C,
            PlayerPosition::FirstBase,
            PlayerPosition::SecondBase,
            PlayerPosition::ThirdBase,
            PlayerPosition::SS,
            PlayerPosition::INF,
            PlayerPosition::LF,
            PlayerPosition::CF,
            PlayerPosition::RF,
            PlayerPosition::OF,
            PlayerPosition::OF,
            PlayerPosition::INF,
        ]
    }

    pub fn get_all_pitcher_positions(era: Era) -> Vec<Self> {
        match era {
            Era::Ancient => vec![PlayerPosition::P],
            Era::Modern => vec![PlayerPosition::SP, PlayerPosition::RP],
        }
    }

    pub fn fix_pos(
        conn: &mut Connection,
        player_id: i64,
        player_name: &str,
        era: Era,
        pd: Option<PD>,
        input: &serde_json::Error,
    ) -> Option<PlayerPosition> {
        println!(
            "There was an error loading the position of player {}, id {}.\n The error was {}",
            player_name, player_id, input
        );

        let user_confirm =
            Confirm::new("Would you like to fix this error? If not, the process will end.")
                .with_default(true)
                .prompt()
                .unwrap_or(false);
        match user_confirm {
            false => None,
            true => {
                let options = match pd {
                    None => PlayerPosition::get_all_batter_positions(),
                    Some(_) => PlayerPosition::get_all_pitcher_positions(era),
                };
                if options.len() == 1 {
                    let new_pos = options.first().unwrap().clone();
                    println!("Setting position to {}", new_pos);
                    Some(new_pos)
                } else {
                    let choice = Select::new("Pick the position for the player.", options).prompt();
                    match choice {
                        Ok(value) => {
                            conn.execute(
                                "UPDATE
                                players
                            SET pos = ?1 
                            WHERE players.player_id = ?2",
                                [value.to_string(), player_id.to_string()],
                            )
                            .unwrap();
                            println!("Setting position to {}", value);
                            Some(value)
                        }
                        Err(_) => None,
                    }
                }
            }
        }
    }
}
