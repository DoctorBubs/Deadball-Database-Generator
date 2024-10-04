use crate::traits::PlayerTrait;
use crate::Contact;
use crate::Defense;
use crate::Deserialize;
use crate::Power;
use crate::Serialize;
use crate::Speed;
use crate::ThreadRng;
use crate::Toughness;
use core::fmt;
use rand::Rng;

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
pub struct LineupInts {
    power: i32,
    contact: i32,
    speed: i32,
}

// BTraits is a struct that contains an instance of all traits that are related to batting, and it represents what batting traits a player has.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct BTraits {
    pub contact: Contact,
    pub defense: Defense,
    pub power: Power,
    pub speed: Speed,
    pub toughness: Toughness,
}

/*This is is a simmilar struct to B_Traits, however instead of an enum each field contains a bool regarding if a trait is above average.
This is used in constructing a lineup. */
pub struct BTraitAboveAverage {
    pub contact: bool,
    pub defense: bool,
    pub power: bool,
    pub speed: bool,
    pub toughness: bool,
}

// To caclulate if a trait is above average, we convert the trait to an int and see if the value is above 0.
fn above_average(b_trait: impl PlayerTrait) -> bool {
    b_trait.to_int() > 0
}

impl BTraits {
    /// Takes a string, and creates a btrait struct via the string.
    /// Traits are separated by commas, and if the traits contradict each other then hte later trait will be used.
    /// Any traits not specified will be the default trait.
    pub fn from_string(input: &str) -> Result<BTraits, String> {
        let mut result = Self::default();
        // We convert the str into an iter, with each value separated by a comma and extra whitespace trimmed.
        let words = input
            // We split the input into multiple strs that are separated by commas.
            .split(',')
            // We trim the whitespace from the str
            .map(|x| x.trim())
            // We filter out strs that are empty.
            .filter(|x| !x.is_empty())
            .map(|x| format!("\"{}\"", x));
        for word in words {
            // If the value is empty, we continue the loop

            // Next, we try to use serde to deserialize a power error
            let power_attempt: Result<Power, serde_json::Error> = serde_json::from_str(&word);
            // Wh check if the power attempt was an error. If not, we assign the to the Btraits struct.
            if let Ok(power) = power_attempt {
                result.power = power;
                continue;
            };
            // We repeat the process for the other traits.
            let contact_attempt: Result<Contact, serde_json::Error> = serde_json::from_str(&word);
            if let Ok(contact) = contact_attempt {
                result.contact = contact;
                continue;
            }
            let defense_attempt: Result<Defense, serde_json::Error> = serde_json::from_str(&word);
            if let Ok(defense) = defense_attempt {
                result.defense = defense;
                continue;
            }
            let speed_attempt: Result<Speed, serde_json::Error> = serde_json::from_str(&word);
            if let Ok(speed) = speed_attempt {
                result.speed = speed;
                continue;
            }
            let toughness_attempt: Result<Toughness, serde_json::Error> =
                serde_json::from_str(&word);
            if let Ok(toughness) = toughness_attempt {
                result.toughness = toughness;
                continue;
            }
            // If we haven''t found a trait that matches the value, we return an error
            let message = format!(
                "Warning: Attempted to parse invalid string '{}' as a trait",
                word
            );
            return Err(message);
        }
        // We return the Btrait struct in an Ok.
        Ok(result)
    }

    pub fn get_above_average(&self) -> BTraitAboveAverage {
        BTraitAboveAverage {
            contact: above_average(self.contact),
            defense: above_average(self.defense),
            power: above_average(self.power),
            speed: above_average(self.speed),
            toughness: above_average(self.toughness),
        }
    }

    pub fn get_trade_value(&self) -> i32 {
        (self.contact.to_int()
            + self.defense.to_int()
            + self.power.to_int()
            + self.speed.to_int()
            + self.toughness.to_int())
            * 5
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
            self.contact, self.defense, self.power, self.speed, self.toughness
        );

        write!(f, "{}", chars)
    }
}
/// Takes a original trait and a new trait. If the new trait is a valid upgrade or downgrade, the new trait is returned.
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
