use std::fmt::Display;

use crate::player;
//use crate::greater_trait;
//use crate::greater_trait as other_greater_trait;
use crate::Deserialize;

use crate::fmt;
use crate::Serialize;

pub trait PlayerTrait {
    fn to_int(&self) -> i32;

    fn get_rbi_score(&self) -> i32 {
        0
    }
}

pub fn trait_to_sql_text<T: PlayerTrait + Display>(player_trait: &T) -> Option<String> {
    let trait_string = player_trait.to_string();
    match trait_string.as_str() {
        "" => None,
        _ => Some(trait_string),
    }
}

// Takes a plert trait. If the result of converting the trait to an int is 0, None is returned, else an option with hte trait is returned.
pub fn player_trait_option<T: PlayerTrait>(player_trait: &T) -> Option<&T> {
    match player_trait.to_int() {
        0 => None,
        _ => Some(player_trait),
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum Power {
    #[serde(rename = "P++")]
    P2,
    #[serde(rename = "P+")]
    P1,
    #[serde(rename = "")]
    P0,
    #[serde(rename = "P-")]
    PM1,
    #[serde(rename = "P--")]
    PM2,
}

//pub fn greater_trait<T: B_Trait>(a: T, b: T) -> T {
//match a.to_int() > b.to_int() {
//  true => a,
// false => b,
//}
//}

impl PlayerTrait for Power {
    fn to_int(&self) -> i32 {
        match self {
            Self::P2 => 2,
            Self::P1 => 1,
            Self::P0 => 0,
            Self::PM1 => -1,
            Self::PM2 => -2,
        }
    }

    fn get_rbi_score(&self) -> i32 {
        self.to_int() * 3
    }
}

impl fmt::Display for Power {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let chars = match self {
            Self::P2 => "P++",
            Self::P1 => "P+",
            Self::P0 => "",
            Self::PM1 => "P-",
            Self::PM2 => "P--",
        };

        write!(f, "{}", chars)
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum Speed {
    #[serde(rename = "S++")]
    S2,
    #[serde(rename = "S+")]
    S1,
    #[serde(rename = "")]
    S0,
    #[serde(rename = "S++")]
    SM1,
}

impl PlayerTrait for Speed {
    fn to_int(&self) -> i32 {
        match self {
            Self::S2 => 2,
            Self::S1 => 1,
            Self::S0 => 0,
            Self::SM1 => -1,
        }
    }
}

impl fmt::Display for Speed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let chars = match self {
            Self::S2 => "S++",
            Self::S1 => "S+",
            Self::S0 => "",
            Self::SM1 => "S-",
        };

        write!(f, "{}", chars)
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum Contact {
    #[serde(rename = "C+")]
    C1,
    #[serde(rename = "")]
    C0,
    #[serde(rename = "C-")]
    CM1,
}

impl PlayerTrait for Contact {
    fn to_int(&self) -> i32 {
        match self {
            Self::C1 => 1,
            Self::C0 => 0,
            Self::CM1 => -1,
        }
    }

    fn get_rbi_score(&self) -> i32 {
        self.to_int() * 2
    }
}

impl fmt::Display for Contact {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let chars = match self {
            Self::C1 => "C+",
            Self::C0 => "",
            Self::CM1 => "C-",
        };

        write!(f, "{}", chars)
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum Defense {
    #[serde(rename = "D+")]
    D1,
    #[serde(rename = "")]
    D0,
    #[serde(rename = "C-")]
    DM1,
}

impl PlayerTrait for Defense {
    fn to_int(&self) -> i32 {
        match self {
            Self::D1 => 1,
            Self::D0 => 0,
            Self::DM1 => -1,
        }
    }
}

impl fmt::Display for Defense {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let chars = match self {
            Self::D1 => "D+",
            Self::D0 => "",
            Self::DM1 => "D-",
        };

        write!(f, "{}", chars)
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]

pub enum Toughness {
    #[serde(rename = "T+")]
    T1,
    #[serde(rename = "")]
    T0,
}

impl PlayerTrait for Toughness {
    fn to_int(&self) -> i32 {
        match self {
            Self::T1 => 1,
            Self::T0 => 0,
        }
    }
}

impl fmt::Display for Toughness {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let chars = match self {
            Self::T1 => "T+",
            Self::T0 => "",
        };

        write!(f, "{}", chars)
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum PitcherTrait {
    #[serde(rename = "CN-")]
    CNM,
    #[serde(rename = "K+")]
    K,
    #[serde(rename = "GB+")]
    GB,
    #[serde(rename = "CN+")]
    CN,
    #[serde(rename = "ST+")]
    ST,
}

impl PlayerTrait for PitcherTrait {
    fn to_int(&self) -> i32 {
        5
    }
}

impl fmt::Display for PitcherTrait {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let chars = match self {
            Self::CNM => "CN-",
            Self::K => "K+",
            Self::GB => "GB",
            Self::CN => "CN+",
            Self::ST => "St+",
        };

        write!(f, "{}", chars)
    }
}
