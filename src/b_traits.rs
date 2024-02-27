use crate::traits::PlayerTrait;
use crate::Contact;
use crate::Defense;
use crate::Power;
use crate::Speed;
use crate::ThreadRng;
use crate::Toughness;
use rand::Rng;

use crate::Deserialize;
use crate::Serialize;
use core::fmt;
fn above_average(b_trait: impl PlayerTrait) -> bool {
    b_trait.to_int() > 0
}

//Creates a struct that lists if a trait is above average

pub struct BTraitAboveAverage {
    pub contact: bool,
    pub defense: bool,
    pub power: bool,
    pub speed: bool,
    pub toughness: bool,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct LineupInts {
    power: i32,
    contact: i32,
    speed: i32,
}

#[derive(Serialize, Deserialize)]
pub struct BTraits {
    pub contact: Contact,
    pub defense: Defense,
    pub power: Power,
    pub speed: Speed,
    pub toughness: Toughness,
}

impl BTraits {
    pub fn get_above_average(&self) -> BTraitAboveAverage {
        BTraitAboveAverage {
            contact: above_average(self.contact),
            defense: above_average(self.defense),
            power: above_average(self.power),
            speed: above_average(self.speed),
            toughness: above_average(self.toughness),
        }
    }
    /*pub fn to_string(&self) -> String {
        format!(
            "{}{}{}{}{}",
            self.contact,
            self.defense,
            self.power,
            self.speed,
            self.toughness
        )
    }*/

    pub fn default() -> BTraits {
        BTraits {
            contact: Contact::C0,
            defense: Defense::D0,
            power: Power::P0,
            speed: Speed::S0,
            toughness: Toughness::T0,
        }
    }

    pub fn get_rbi_score(&self) -> i32 {
        self.contact.get_rbi_score() + self.power.get_rbi_score()
    }

    pub fn generate(&mut self, thread: &mut ThreadRng) -> bool {
        let roll = thread.gen_range(1..=10) + thread.gen_range(1..=10);

        match roll {
            2 => {
                self.power = trait_stack(self.power, Power::PM2);
                true
            }
            3 => {
                self.power = trait_stack(self.power, Power::PM1);
                true
            }
            4 => {
                self.speed = trait_stack(self.speed, Speed::SM1);
                true
            }
            5 => {
                self.contact = trait_stack(self.contact, Contact::CM1);
                true
            }
            6 => {
                self.defense = trait_stack(self.defense, Defense::DM1);
                true
            }
            15 => {
                self.defense = trait_stack(self.defense, Defense::D1);
                true
            }
            16 => {
                self.power = trait_stack(self.power, Power::P1);
                true
            }
            17 => {
                self.contact = trait_stack(self.contact, Contact::C1);
                true
            }
            18 => {
                self.speed = trait_stack(self.speed, Speed::S1);
                true
            }
            19 => {
                self.toughness = trait_stack(self.toughness, Toughness::T1);
                true
            }
            20 => {
                self.power = trait_stack(self.power, Power::P2);
                true
            }
            _ => false,
        }
    }

    /*
    fn trait_swap(&self,new_trait: impl B_Trait){

      match new_trait.trait_string(){

        "Power" =>{

          self.power = trait_swap(self.power,new_trait)
        }


      }


    }
    */
}


impl fmt::Display for BTraits {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let chars = format!(
            "{}{}{}{}{}",
            self.contact,
            self.defense,
            self.power,
            self.speed,
            self.toughness
        );
    

        write!(f,"{}", chars)
    }
}

fn trait_stack<T: PlayerTrait>(original: T, new: T) -> T {
    let original_num = original.to_int();
    let new_num = new.to_int();

    let tup = (
        (original_num == 0) & (new_num != 0),
        (original_num > 0) & (new_num > original_num),
        (original_num < 0) & (new_num < original_num),
    );
    match tup {
        (false, false, false) => original,
        _ => new,
    }
}
