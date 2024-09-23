use std::thread;

use crate::b_traits::BTraits;
use crate::note::Note;
use crate::pd::PD;
use crate::player::Hand;
use crate::player::Player;
use crate::traits::Contact;
use crate::traits::PitcherTrait;
use crate::Era;
use crate::ThreadRng;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;

/*  There are several fields on the player struct that are generated differently for batters and pitchers. We use a PlayerQuality trait to summarize what calculation have to be made for both,
    and we apply the traits to an enum for both pitchers and hittters to determnig the calculations for each.
*/


pub trait PlayerQuality {
    fn get_bt(&self, thread: &mut ThreadRng) -> i32;
    fn get_obt_mod(&self, thread: &mut ThreadRng) -> i32;
    fn get_pd(&self, thread: &mut ThreadRng, era: Era) -> Option<PD>;
    fn for_pitcher(&self) -> bool;
    fn get_hand(&self, thread: &mut ThreadRng) -> Hand {
        let roll = thread.gen_range(1..=10);
        match roll {
            1..=6 => Hand::R,
            7..=9 => Hand::L,
            10 => match self.for_pitcher() {
                true => Hand::L,
                false => Hand::S,
            },
            _ => Hand::R,
        }
    }
    fn calc_traits(&self, _trait_struct: &mut BTraits, _thread: &mut ThreadRng) {}
    fn get_pitcher_trait(&self, thread: &mut ThreadRng) -> Option<PitcherTrait>;
    fn upgrade(&self) -> Self;
    fn get_box_copy(&self) -> Box<Self>;
    /// Randomly generates a base player based off the quality.
    fn gen_player(&self, thread: &mut ThreadRng, era: Era) -> Player {
        let bt = self.get_bt(thread);
        let obt_mod = self.get_obt_mod(thread);
        let obt = bt + obt_mod;
        let mut b_traits = BTraits::default();
        self.calc_traits(&mut b_traits, thread);
        let pd = self.get_pd(thread, era);
        let pitcher_trait = self.get_pitcher_trait(thread);
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
        let hand = self.get_hand(thread);
        let (age, pos, name, player_id, team_id, note) = Player::get_default_info();
        Player {
            name,
            age,
            pos,
            bt,
            obt_mod,
            obt,
            b_traits,
            pd,
            pitcher_trait,
            trade_value,
            hand,
            player_id,
            team_id,
            note
        }
    }
}

/* Batter quality is the enum used ot generated batters. Batters do not get a base pitch die, however their stats for hitting are much better then pitchers.
 The batter quality enum has 2 levels, TopProspect and Farmhand. Currently, TopProspect is used to generate players in a teams starting lineup, while the lower quality
farmhands is used for the typically worse performing bench players. */
#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum BatterQuality {
    TopProspect,
    Farmhand,
}

fn new_bt(die: i32, base: i32, thread: &mut ThreadRng) -> i32 {
    let mut result = base;

    for _i in 1..=2 {
        result += thread.gen_range(1..=die);
    }
    result
}

impl PlayerQuality for BatterQuality {
    fn get_bt(&self, thread: &mut ThreadRng) -> i32 {
        match self {
            Self::TopProspect => new_bt(10, 15, thread),
            Self::Farmhand => new_bt(10, 12, thread),
        }
    }
    fn get_pitcher_trait(&self, _thread: &mut ThreadRng) -> Option<PitcherTrait> {
        None
    }
    fn get_obt_mod(&self, thread: &mut ThreadRng) -> i32 {
        thread.gen_range(1..=4) + thread.gen_range(1..=4)
    }

    fn get_pd(&self, _threat: &mut ThreadRng, _era: Era) -> Option<PD> {
        None
    }

    fn for_pitcher(&self) -> bool {
        false
    }

    fn calc_traits(&self, trait_struct: &mut BTraits, thread: &mut ThreadRng) {
        let first_calc = trait_struct.generate(thread);
        if first_calc {
            if let BatterQuality::TopProspect = self {
                trait_struct.generate(thread);
            }
        }
    }

    fn upgrade(&self) -> Self {
        Self::TopProspect
    }
    fn get_box_copy(&self) -> Box<Self> {
        Box::new(*self)
    }
}

// The pitcher quality enum is used to generate the pitcher stats. Currenly, only the top prosepect enum is used, this may change in the future.
#[derive(Copy, Clone)]
pub enum PitcherQuality {
    TopProspect,
    Farmhand,
}

impl PlayerQuality for PitcherQuality {
    fn get_bt(&self, thread: &mut ThreadRng) -> i32 {
        new_bt(6, 12, thread)
    }

    fn get_obt_mod(&self, thread: &mut ThreadRng) -> i32 {
        thread.gen_range(1..=8)
    }
    // A players PD is generated based off the current eta
    fn get_pd(&self, thread: &mut ThreadRng, era: Era) -> Option<PD> {
        let new_pd = era.new_pd(thread, self);
        Some(new_pd)
    }

    fn for_pitcher(&self) -> bool {
        true
    }

    fn get_pitcher_trait(&self, thread: &mut ThreadRng) -> Option<PitcherTrait> {
        let roll = thread.gen_range(1..=10) + thread.gen_range(1..=10);
        //println!("Pitch Trait roll = {}", roll);
        match roll {
            5 => Some(PitcherTrait::CNM),
            15 => Some(PitcherTrait::K),
            16 => Some(PitcherTrait::GB),
            17 => Some(PitcherTrait::CN),
            18 => Some(PitcherTrait::ST),
            _ => None,
        }
    }

    fn upgrade(&self) -> Self {
        Self::TopProspect
    }

    fn get_box_copy(&self) -> Box<Self> {
        Box::new(*self)
    }
}

impl PitcherQuality {
    pub fn get_pd_modifier(&self) -> i32 {
        match self {
            Self::TopProspect => 0,
            Self::Farmhand => 2,
        }
    }
}
