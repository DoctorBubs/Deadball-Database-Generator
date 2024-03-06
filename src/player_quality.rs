use crate::b_traits::BTraits;
use crate::pd::PD;
use crate::traits::PitcherTrait;
use crate::Era;
use crate::ThreadRng;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;

// Batters and Pitchers both have  several calculations to be common, howver the way their stats are generated differ.
pub trait PlayerQuality {
    fn get_bt(&self, thread: &mut ThreadRng) -> i32;
    fn get_obt_mod(&self, thread: &mut ThreadRng) -> i32;
    fn get_pd(&self, thread: &mut ThreadRng, era: Era) -> Option<PD>;
    fn for_pitcher(&self) -> bool;

    fn calc_traits(&self, _trait_struct: &mut BTraits, _thread: &mut ThreadRng) {}
    fn get_pitcher_trait(&self, thread: &mut ThreadRng) -> Option<PitcherTrait>;
}

/* Batterquality is the enum used ot generated batters. Batters do not get a base pitch die, however their stats for hitting are much better then pitchers.
 The batter qualtiy enum has 2 levels, TopProspect adn Framhand. Currently, TopProspect is used to generatee playts in a teams startinging lineup, while the lower quality
farmhands is used for bench players */
#[derive(Copy, Clone,Serialize,Deserialize)]
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
}

// The pitcher quality enum is used to generate the pitcher stat. Currenly, only the top prosepect enum is ued.
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
}

impl PitcherQuality {
    pub fn get_pd_modifier(&self) -> i32 {
        match self {
            Self::TopProspect => 0,
            Self::Farmhand => 2,
        }
    }
}
