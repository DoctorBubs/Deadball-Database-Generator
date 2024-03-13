use crate::Deserialize;
use crate::PitcherQuality;
use crate::Serialize;
use crate::ThreadRng;
use crate::PD;
use core::fmt;
use inquire::InquireError;
use inquire::Select;
use rand::Rng;
/* Deadball has 2 sets of rules to simulate 2 different era's of baseball.
The Ancient Era simulates the low scoring style of basbeall playedf in the early 1900's, while the modern is used to simulate baseball since.
The main difference is how the the pitch die for pitchers is generated, however it also influence the numbers of players on the roster, as well as player postions.*/
#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Era {
    Ancient,
    Modern,
}
impl Era {
    pub fn new_pd(&self, thread: &mut ThreadRng, quality: &PitcherQuality) -> PD {
        match self {
            // To siumlate the low scoring offense of 1900/s baseball, Anicnet Era pitchers have a signifacntly higher cieling, as their base pitch die can be as high as a D20
            Self::Ancient => {
                let roll = thread.gen_range(1..=12) + quality.get_pd_modifier();
                match roll {
                    1 => PD::D20,
                    2..=3 => PD::D12,
                    4..=5 => PD::D8,
                    6..=8 => PD::D6,
                    9..=10 => PD::D4,
                    11 => PD::D0,
                    12..=13 => PD::DM4,
                    _ => PD::DM8,
                }
            }
            // Modern era pitchers get a siginifacnly lowert cieling for the pd, with a d12 being the higherst  base pd generated.
            Self::Modern => {
                let roll = thread.gen_range(1..=8) + quality.get_pd_modifier();
                match roll {
                    1 => PD::D12,
                    2..=3 => PD::D8,
                    4..=7 => PD::D4,
                    _ => PD::DM4,
                }
            }
        }
    }
}

impl fmt::Display for Era {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let chars = match self {
            Self::Modern => "Modern",
            Self::Ancient => "Ancient",
        };

        write!(f, "{}", chars)
    }
}

//Prompts a user to select  new era.
pub fn select_era() -> Era {
    let options: Vec<Era> = vec![Era::Ancient, Era::Modern];
    let ans: Result<Era, InquireError> =
        Select::new("Select the era for the league", options).prompt();
    match ans {
        Ok(era) => era,
        Err(_) => panic!("Error selecting era"),
    }
}