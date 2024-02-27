use rand::rngs::ThreadRng;
use rand::Rng;
use std::collections::HashMap;
mod pd;
use crate::pd::PD;

mod on_base;
mod traits;
use crate::traits::Contact;
use crate::traits::Defense;

use crate::traits::Power;
use crate::traits::Speed;
use crate::traits::Toughness;
mod b_traits;
mod lineup_score;
mod pitcher_rank_info;
mod player;
mod player_quality;
use crate::player::Player;
//use crate::traits::greater_trait;

use crate::player::PlayerGender;
use crate::player_quality::BatterQuality;
use crate::player_quality::PitcherQuality;
use crate::player_quality::PlayerQuality;

mod team;
use crate::team::Team;

mod league;
use crate::league::League;

use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

use serde::{Deserialize, Serialize};

/* Deadball has 2 sets of rules to simulate 2 different era's of baseball.
The Ancient Era simulates the low scoring style of basbeall playedf in the early 1900's, while the modern is used to simulate baseball since.
The main difference is how the the pitch die for pitchers is generated, however it also influence the numbers of players on the roster, as well as player postions.*/
#[derive(Copy, Clone, Serialize, Deserialize)]
enum Era {
    Ancient,
    Modern,
}

impl Era {
    fn new_pd(&self, thread: &mut ThreadRng, quality: &PitcherQuality) -> PD {
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

#[derive(Serialize, Deserialize)]
struct BattingStats {
    counting_stats: HashMap<String, i32>,
}

fn trimed_capital_input() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let result = input.trim().to_uppercase();
    result
}

fn get_y_n() -> bool {
    loop {
        match trimed_capital_input().as_str() {
            "Y" => return true,
            "N" => return false,
            _ => {
                println!("Invalid Input, valid input is y, Y, n, or N")
            }
        }
    }
}

//#[tailcall]
fn select_era() -> Era {
    let result: Era;
    loop {
        println!("Press A to have the era for the league be ancient, press M to have the leage be modern");
        let mut era_input = String::new();
        io::stdin()
            .read_line(&mut era_input)
            .expect("Failed to read line");
        let era_name = era_input.trim().to_uppercase();
        match era_name.as_str() {
            "A" => {
                result = Era::Ancient;
                break;
            }
            "M" => {
                result = Era::Modern;
                break;
            }
            _ => {
                println!("Invalid Era")
            }
        };
    }
    result
}
//#[tailcall]
fn select_gender() -> PlayerGender {
    let result: PlayerGender;
    loop {
        println!("Press M for the League Gender to be male, F for Female, C for Coed");
        let mut gender_input = String::new();
        io::stdin()
            .read_line(&mut gender_input)
            .expect("Failed to read line");
        let gender_name = gender_input.trim().to_uppercase();
        match gender_name.as_str() {
            "M" => {
                result = PlayerGender::Male;
                break;
            }
            "F" => {
                result = PlayerGender::Female;
                break;
            }
            "C" => {
                result = PlayerGender::Coed;
                break;
            }
            _ => {
                println!("Invalid Gender");
            }
        }
    }

    result
}

fn create_new_league(thread: &mut ThreadRng) -> std::io::Result<()> {
    println!("Enter the name of the new league");
    let mut league_input = String::new();
    io::stdin()
        .read_line(&mut league_input)
        .expect("Failed to read line");

    let league_name = league_input.trim().to_string();

    let folder_path = Path::new(&league_name);

    fs::create_dir(folder_path)?;

    let era = select_era();

    let gender = select_gender();

    let mut new_league = League::new(&league_name, gender, era);

    add_new_team(&mut new_league, folder_path, thread, true)
}

//#[tailcall]
fn add_new_team(
    league: &mut League,
    path: &Path,
    thread: &mut ThreadRng,
    first_team: bool,
) -> std::io::Result<()> {
    let result: std::io::Result<()>;
    loop {
        match first_team {
            true => println!("Enter the name of the first team"),
            false => println!("Enter the name of the new team"),
        };
        let mut team_input = String::new();
        io::stdin()
            .read_line(&mut team_input)
            .expect("Failed to read line");
        let team_name = team_input.trim().to_string();
        let team_path = path.join(format!("{}.txt", team_name));
        match team_path.exists() {
            true => {
                println!("A team with that name already exists for this leauge");
            }

            false => {
                let team = league.new_team(&team_name, thread);
                let mut team_info = File::create(team_path)?;
                team_info.write_all(team.to_string().as_bytes())?;
                league.add_team(team);
                result = add_team_check(league, path, thread);
                break;
            }
        }
    }
    result
}
//#[tailcall]
fn add_team_check(league: &mut League, path: &Path, thread: &mut ThreadRng) -> std::io::Result<()> {
    println!("Would you like to create another team? Y/N");

    /*let mut check_input = String::new();

    io::stdin()
        .read_line(&mut check_input)
        .expect("Failed to read line");
    let check = check_input.trim().to_uppercase().to_string(); */

    match get_y_n() {
        true => add_new_team(league, path, thread, false),
        false => save_league(league, path),
    }
}

// Take's a league and a path, serializes tje league to a json object, which is saved under path.league_info.text
fn save_league(league: &League, path: &Path) -> std::io::Result<()> {
    let serial = serde_json::to_string(&league).unwrap();
    let new_file_path = path.join("league_info.txt");

    let mut league_info = File::create(new_file_path)?;
    league_info.write_all(serial.as_bytes())?;
    Ok(())
}

fn get_league_name() -> String {
    println!("Enter the name of the league you would like to add a team to.");
    let mut check_input = String::new();

    io::stdin()
        .read_line(&mut check_input)
        .expect("Failed to read line");
    check_input.trim().to_string()
}

//#[tailcall]
fn league_check(thread: &mut ThreadRng) -> std::io::Result<()> {
    let result: std::io::Result<()>;
    loop {
        let folder_name = get_league_name();
        let path = Path::new(&folder_name);
        match path.exists() {
            true => {
                result = load_league(thread, path);
                break;
            }
            false => {
                println!("Can not find folder with that name");

                println!("Would you like to load a different league? Y/N");
                match get_y_n() {
                    false => {
                        result = Ok(());
                        break;
                    }
                    true => (),
                };
            }
        };
    }

    result
}

fn load_league(thread: &mut ThreadRng, path: &Path) -> std::io::Result<()> {
    let mut league: League;
    let league_info =
        fs::read_to_string(path.join("league_info.txt")).expect("league_info file is missing");
    let league_result = serde_json::from_str(&league_info);

    match league_result {
        Ok(loaded_league) => league = loaded_league,
        Err(_) => {
            println!("League Info file is corrupted!");
            loop {
                println!("Would you like to load a different league? Y / N");
                match trimed_capital_input().as_str() {
                    "N" => return Ok(()),
                    "Y" => return league_check(thread),
                    _ => println!("Invalid Input"),
                }
            }
        }
    };

    println!("League Loaded");
    add_team_check(&mut league, path, thread)
}

//#[tailcall]
fn main() -> std::io::Result<()> {
    let mut r_thread = rand::thread_rng();
    loop {
        println!("Press l to create a new league, t to create a new team");
        /*let mut first_input = String::new();

        io::stdin()
            .read_line(&mut first_input)
            .expect("Invalid Input");


        let input_str = first_input.trim()
            .to_uppercase();

        */
        match trimed_capital_input().as_str() {
            "L" => return create_new_league(&mut r_thread),
            "T" => return league_check(&mut r_thread),
            _ => {
                println!("Invalid Input");
            }
        };
    }
}
