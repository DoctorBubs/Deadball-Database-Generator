use crate::b_traits::BTraitAboveAverage;
use crate::b_traits::BTraits;
use crate::league::EditLeagueError;
use crate::lineup_score::LineupScore;
use crate::pitcher_rank_info::PitcherRankInfo;
use crate::player_quality::PlayerQuality;
use crate::player_serde::PlayerSerde;
use crate::team::TeamSpot;
use crate::traits::player_trait_option;
use crate::traits::PitcherTrait;
use crate::Deserialize;
use crate::Era;
use crate::Serialize;
use crate::PD;
use inquire::InquireError;
use inquire::Select;
use name_maker::Gender;
use name_maker::RandomNameGenerator;
use rand::rngs::ThreadRng;
use rand::Rng;
use rusqlite::Connection;

use std::fmt;
pub enum AgeCat {
    Prospect,
    Rookie,
    Veteran,
    OldTimer,
}

impl AgeCat {
    pub fn random(thread: &mut ThreadRng) -> AgeCat {
        let roll = thread.gen_range(1..=6);
        match roll {
            1..=2 => Self::Prospect,
            3..=4 => Self::Rookie,
            5 => Self::Veteran,
            6 => Self::OldTimer,
            _ => Self::Rookie,
        }
    }

    pub fn new_age(&self, thread: &mut ThreadRng) -> i32 {
        let roll = thread.gen_range(1..=6);
        match self {
            Self::Prospect => 18 + roll,
            Self::Rookie => 21 + roll,
            Self::Veteran => 26 + roll,
            Self::OldTimer => 32 + roll,
        }
    }
}

// Players can be either left handed or right handed, however batters may also be switch hitters. We use an enum to keep track.
#[derive(Serialize, Deserialize, Debug)]
pub enum Hand {
    R,
    L,
    S,
}

impl Hand {
    pub fn new(thread: &mut ThreadRng, quality: &impl PlayerQuality) -> Hand {
        let roll = thread.gen_range(1..=10);
        match roll {
            1..=6 => Self::R,
            7..=9 => Self::L,
            10 => match quality.for_pitcher() {
                true => Self::L,
                false => Self::S,
            },
            _ => Self::R,
        }
    }
}

impl fmt::Display for Hand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let chars = match self {
            Self::R => "R",
            Self::L => "L",
            Self::S => "S",
        };

        write!(f, "{}", chars)
    }
}

// Player gender is merely cosmetic, as it is only used to generate a name for the player.
#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum PlayerGender {
    Male,
    Female,
    Coed,
}

impl PlayerGender {
    pub fn new_name(&self) -> String {
        let generator = RandomNameGenerator::init();
        match self {
            Self::Male => generator.generate_specific(Gender::Male).to_string(),
            Self::Female => generator.generate_specific(Gender::Female).to_string(),
            Self::Coed => generator.generate().to_string(),
        }
    }
}

impl fmt::Display for PlayerGender {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let chars = match self {
            Self::Male => "Male",
            Self::Female => "Female",
            Self::Coed => "Coed",
        };

        write!(f, "{}", chars)
    }
}
// Prompts a user to select a player gender
pub fn select_gender() -> Result<PlayerGender, InquireError> {
    let options: Vec<PlayerGender> =
        vec![PlayerGender::Male, PlayerGender::Female, PlayerGender::Coed];

    Select::new("Select the league gender,", options).prompt()
}

#[derive(Serialize, Deserialize, Debug)]

pub struct Player {
    pub name: String,
    pub age: i32,
    pub pos: String,
    pub hand: Hand,
    pub bt: i32,        // BT is essentially a players batting average.
    pub obt_mod: i32,   // Used to calculate a players obt via summing with it's bt.'
    pub obt: i32,       // A player's obt is calculated by adding its bt + its obt_mod
    pub pd: Option<PD>, // The main difference between a batter and pitcher is that pitchers have a base pitch die associated with themselves, while batters do not.
    // This is simulated using an option.
    pub b_traits: BTraits,
    pub pitcher_trait: Option<PitcherTrait>,
    pub trade_value: i32,
    pub team_id: i64,
    pub player_id: i64,
}

impl Player {
    pub fn get_base_pd(&self) -> PD {
        self.pd.unwrap_or(PD::DM20)
        //let die = self.pd.unwrap_or(PD::DM20);
        //die.clone()
    }

    // Determine's if a player is a pitcher based off if it has a pitch die or not.'
    pub fn is_pitcher(&self) -> bool {
        matches!(self, Player { pd: Some(_), .. })

        /*match self.pd {
            Some(pd) => true,
            None => false,
        }*/
    }
    /* For lineup purposes, this calculates if a player should be batting at or near the top of the lineup.
    This is mostly based off a players on base target, with a slight preference to players with low power and speed */

    pub fn get_leadoff_score(&self) -> i32 {
        match self.b_traits.get_above_average() {
            BTraitAboveAverage {
                speed: true,
                power: false,
                contact: false,
                ..
            } => self.obt + 2,
            BTraitAboveAverage { power: true, .. } | BTraitAboveAverage { contact: true, .. } => {
                self.obt - 2
            }
            _ => self.obt,
        }
    }

    // We also calculate how good a player would be at getting rbi.
    // higher bt and positive power and contact traits are preferred.
    pub fn get_rbi_score(&self) -> i32 {
        self.bt + self.b_traits.get_rbi_score()
    }

    pub fn get_lineup_score(&self) -> LineupScore {
        LineupScore {
            leadoff_score: self.get_leadoff_score(),
            rbi_score: self.get_rbi_score(),
            switch_hitter_bonus: match self.hand {
                Hand::S => 1,
                _ => 0,
            },
            string: self.to_string(),
        }
    }

    fn get_pitcher_trait_string(&self) -> String {
        match self.pitcher_trait {
            Some(pitcher_trait) => format!("{},", pitcher_trait),
            None => "".to_string(),
        }
    }

    pub fn get_team_score_contribution(&self) -> i32 {
        match self.is_pitcher() {
            true => self.get_base_pd().to_int(),
            false => self.bt,
        }
    }

    pub fn get_serde(&mut self, team_spot: TeamSpot) -> Result<PlayerSerde, serde_json::Error> {
        //  self.team_id = team_id;
        let BTraits {
            contact,
            speed,
            power,
            toughness,
            defense,
        } = &self.b_traits;

        let contact_option = player_trait_option(contact);
        let defense_option = player_trait_option(defense);
        let power_option = player_trait_option(power);
        let speed_option = player_trait_option(speed);
        let toughness_option = player_trait_option(toughness);

        let pd_int = self.pd.map(|die| die.to_int());

        let new_player_serde = PlayerSerde {
            team_id: self.team_id,
            player_name: &self.name,
            pos: &self.pos,
            age: self.age,
            hand: serde_json::to_value(&self.hand)?,
            bt: self.bt.to_string(),
            obt_mod: self.obt_mod.to_string(),
            obt: self.obt.to_string(),
            pd: serde_json::to_value(self.pd)?,
            pd_int: serde_json::to_value(pd_int)?,
            pitcher_trait: serde_json::to_value(self.pitcher_trait)?,
            team_spot: serde_json::to_string(&team_spot)?,
            contact: serde_json::to_value(contact_option)?,
            defense: serde_json::to_value(defense_option)?,
            power: serde_json::to_value(power_option)?,
            speed: serde_json::to_value(speed_option)?,
            toughness: serde_json::to_value(toughness_option)?,
            trade_value: self.trade_value,
        };

        Ok(new_player_serde)
    }
    pub fn save_sql(
        &mut self,
        conn: &mut Connection,
        team_id: i64,
        team_spot: TeamSpot,
    ) -> Result<(), EditLeagueError> {
        self.team_id = team_id;
        let p_serde = match self.get_serde(team_spot) {
            Ok(data) => data,
            Err(message) => return Err(EditLeagueError::SerdeError(message)),
        };

        let new_player_id = match p_serde.save_to_sql(conn) {
            Ok(num) => num,
            Err(message) => return Err(EditLeagueError::DatabaseError(message)),
        };
        self.player_id = new_player_id;
        Ok(())
    }
    // Used to rank pitchers.
    pub fn get_pitcher_rank_info(&self) -> PitcherRankInfo {
        let pd_num = self.get_base_pd().to_int();
        PitcherRankInfo {
            num: match self.pitcher_trait {
                Some(_) => pd_num + 1,
                None => pd_num,
            },
            age: self.age,
            string: self.to_string(),
        }
    }

    pub fn new(
        pos: String,
        gender: PlayerGender,
        quality: impl PlayerQuality,
        thread: &mut ThreadRng,
        era: Era,
    ) -> Player {
        let name = gender.new_name();
        let bt = quality.get_bt(thread);
        let obt_mod = quality.get_obt_mod(thread);
        let obt = bt + obt_mod;
        let mut b_traits = BTraits::default();
        quality.calc_traits(&mut b_traits, thread);
        /*let pd = match quality{
        BatterQuality::Pitcher => Some(PD::D8),
        _ => None
        }*/
        let pd = quality.get_pd(thread, era);
        let pitcher_trait = quality.get_pitcher_trait(thread);
        let hand = Hand::new(thread, &quality);
        let age_cat = AgeCat::random(thread);
        let age = age_cat.new_age(thread);
        let trade_value = match pd {
            None => bt + b_traits.get_trade_value(),
            Some(pd) => {
                let base = pd.to_int();
                match pitcher_trait {
                    Some(_) => (base + 1) * 5,
                    None => base * 5,
                }
            }
        };
        // Since we don't know what the team_id or player_id will be in the database, we set both to 0 temporarily.
        let team_id = 0;
        let player_id = 0;
        Player {
            name,
            pos,
            age,
            hand,
            bt,
            obt_mod,
            obt,
            pd,
            b_traits,
            pitcher_trait,
            trade_value,
            team_id,
            player_id,
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let chars = match self.is_pitcher() {
            false => {
                let base = format!(
                    "{},{},{},{},{},{}",
                    self.name, self.pos, self.age, self.hand, self.bt, self.obt
                );
                let trait_string = self.b_traits.to_string();
                match trait_string.trim().is_empty() {
                    true => base,
                    false => format!("{},{}", base, trait_string),
                }
            }

            true => {
                format!(
                    "{},{},{},{},{},{} {},{}",
                    self.name,
                    self.pos,
                    self.age,
                    self.hand,
                    self.get_base_pd(),
                    self.get_pitcher_trait_string(),
                    self.bt,
                    self.obt
                )
            }
        };

        write!(f, "{}", chars)
    }
}
