//use crate::greater_trait;
//use crate::greater_trait as other_greater_trait;
use crate::Deserialize;

use crate::Serialize;

pub trait PlayerTrait {
    fn to_int(&self) -> i32;
    fn to_string(&self) -> String;
    fn get_rbi_score(&self) -> i32{
        0
    }
    //fn upgrade<T>(&self) -> Option<T>;
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Power {
    P2,
    P1,
    P0,
    PM1,
    PM2,
}

// Returns true if a newer trait can replace an older trai during trait generation
// For exampple, P++ can replace P+
pub fn replace_trait<T: PlayerTrait>(older: &T, newer: &T) -> bool {
    match older.to_int() == 0 {
        true => true,
        false => {
            match (
                older.to_int() >= 0,
                newer.to_int() >= 0,
                newer.to_int() > older.to_int(),
            ) {
                (true, true, true) => true,
                (false, false, false) => true,
                _ => false,
            }
        }
    }
}

pub fn trait_plus(a: impl PlayerTrait, b: impl PlayerTrait) -> i32 {
    a.to_int() + b.to_int()
}

//pub fn greater_trait<T: B_Trait>(a: T, b: T) -> T {
//match a.to_int() > b.to_int() {
//  true => a,
// false => b,
//}
//}

pub fn trait_swap<T: PlayerTrait>(a: T, b: T) -> T {
    match a.to_int() >= b.to_int() {
        true => a,
        false => b
    }
}

impl PlayerTrait for Power {
    // fn upgrade<Power>(&self) -> Option<Power>{

    //match self{

    // Self::P1 => Some(Self::P2),
    // _ => None

    //  }

    //}

    fn to_string(&self) -> String {
        match self {
            Self::P2 => "P++".to_string(),
            Self::P1 => "P+".to_string(),
            Self::P0 => "".to_string(),
            Self::PM1 => "P-".to_string(),
            Self::PM2 => "P--".to_string(),
        }
    }

    fn to_int(&self) -> i32 {
        match self {
            Self::P2 => 2,
            Self::P1 => 1,
            Self::P0 => 0,
            Self::PM1 => -1,
            Self::PM2 => -2,
        }
    }

    fn get_rbi_score(&self) -> i32{
        self.to_int() * 3
    
    }
    
}
#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Speed {
    S2,
    S1,
    S0,
    SM1,
}

impl PlayerTrait for Speed {
    fn to_string(&self) -> String {
        match self {
            Self::S2 => "S++".to_string(),
            Self::S1 => "S+".to_string(),
            Self::S0 => "".to_string(),
            Self::SM1 => "S-".to_string(),
        }
    }

    fn to_int(&self) -> i32 {
        match self {
            Self::S2 => 2,
            Self::S1 => 1,
            Self::S0 => 0,
            Self::SM1 => -1,
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Contact {
    C1,
    C0,
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

    fn to_string(&self) -> String {
        match self {
            Self::C1 => "C+".to_string(),
            Self::C0 => "".to_string(),
            Self::CM1 => "C-".to_string(),
        }
    }

    fn get_rbi_score(&self) -> i32{
    
        self.to_int() * 2
    
    }
}
#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Defense {
    D1,
    D0,
    DM1,
}

impl PlayerTrait for Defense {
    fn to_string(&self) -> String {
        match self {
            Self::D1 => "D+".to_string(),
            Self::D0 => "".to_string(),
            Self::DM1 => "D-".to_string(),
        }
    }

    fn to_int(&self) -> i32 {
        match self {
            Self::D1 => 1,
            Self::D0 => 0,
            Self::DM1 => -1,
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]

pub enum Toughness {
    T1,
    T0,
}

impl PlayerTrait for Toughness {
    fn to_string(&self) -> String {
        match self {
            Self::T1 => "T+".to_string(),
            Self::T0 => "".to_string(),
        }
    }

    fn to_int(&self) -> i32 {
        match self {
            Self::T1 => 1,
            Self::T0 => 0,
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum PitcherTrait {
    CNM,
    K,
    GB,
    CN,
    ST,
}

impl PlayerTrait for PitcherTrait {
    fn to_int(&self) -> i32 {
        5
    }

    fn to_string(&self) -> String {
        match self {
            Self::CNM => "CN-".to_string(),
            Self::K => "K+".to_string(),
            Self::GB => "GB".to_string(),
            Self::CN => "CN+".to_string(),
            Self::ST => "St+".to_string(),
        }
    }
}
