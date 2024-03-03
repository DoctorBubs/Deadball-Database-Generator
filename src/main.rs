mod b_traits;
mod era;
mod league;
mod lineup_score;
mod pd;
mod pitcher_rank_info;
mod player;
mod player_quality;
mod team;
mod traits;
use crate::era::Era;
use crate::league::League;
use crate::pd::PD;
use crate::player::Player;
use crate::player::PlayerGender;
use crate::player_quality::BatterQuality;
use crate::player_quality::PitcherQuality;
use crate::player_quality::PlayerQuality;
use crate::team::Team;
use crate::league::AddTeamError;
use crate::traits::Contact;
use crate::traits::Defense;
use crate::traits::Power;
use crate::traits::Speed;
use crate::traits::Toughness;
use crate::validator::MinLengthValidator;
use crate::validator::MaxLengthValidator;
use inquire::*;
use rand::rngs::ThreadRng;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::rc::Rc;
use std::path::PathBuf;


fn trimed_capital_input() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let result = input.trim().to_uppercase();
    result
}

//#[tailcall]
fn select_era() -> Era {
    let options: Vec<Era> = vec![Era::Ancient, Era::Modern];
    let ans: Result<Era, InquireError> =
        Select::new("Select the era for the league", options).prompt();
    match ans {
        Ok(era) => era,
        Err(_) => panic!("Error selecting era"),
    }
}
//#[tailcall]
fn select_gender() -> PlayerGender {
    let options: Vec<PlayerGender> =
        vec![PlayerGender::Male, PlayerGender::Female, PlayerGender::Coed];
    let ans: Result<PlayerGender, InquireError> =
        Select::new("Select the league gender,", options).prompt();
    match ans {
        Ok(gender) => gender,
        Err(_) => panic!("Error selecting gender"),
    }
}

/*struct LeagueEntry<'a>{

    name: String,
    path: &'a Path

}

impl LeagueEntry<'_>{

    fn new() -> LeagueEntry<'static>{
        let validator = MinLengthValidator::new(3);
        loop{
            let league_input = Text::new("Enter the name for the new league")
                .with_validator(validator.clone())
                .prompt();
            let name = match league_input {
                Ok(input) => input.trim().to_string(),
                Err(_) => panic!("Error creating a league name"),
            };
            let path = Path::new(&name);

            match fs::create_dir(path){
            
                Ok(_) => return LeagueEntry{
                    name: name,
                    path: &path
                
                },

                Err(_) => ()
            
            
            }    
        }
    }

} */

fn create_new_league(thread: &mut ThreadRng) -> std::io::Result<()> {
    let mut league_name: String;
    let mut folder_path: &Path;
    let validator = MinLengthValidator::new(3);
    loop{
        let league_input = Text::new("Enter the name for the new league")
            .with_validator(validator.clone())
            .prompt();
        league_name = match league_input {
            Ok(input) => input.trim().to_string(),
            Err(_) => panic!("Error creating a league name"),
        };
        
        folder_path = Path::new(&league_name);
        
        match fs::create_dir(folder_path){
            Ok(_) => break,
            Err(_) => println!("Error creating a new league folder. Perhaps there is already a league with the same name?")
        
        }
    };
    //let LeagueEntry{name: league_name,path: folder_path} = LeagueEntry::new();
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

    let mut prompt_string = match first_team {
        true => "Enter the name of the first team",
        false => "Enter the name of the new team",
    };

    let abrv_min_validator = MinLengthValidator::new(2);
    let abrv_max_validator = MaxLengthValidator::new(4);
    let name_validator = MinLengthValidator::new(3);
    loop {
        let name_input = Text::new(prompt_string)
            .with_validator(name_validator.clone())
            .prompt();

        let team_name = match name_input {
            Ok(name) => name.trim().to_string(),
            Err(_) => panic!("Error creating team name"),
        };

        let abrv_input = Text::new("Please enter an abbreviation for the new team")
            .with_validator(abrv_min_validator.clone())
            .with_validator(abrv_max_validator.clone())
            .with_default(&team_name[0..=1].to_string())
            .prompt();
    
        let abrv = match abrv_input{
        
            Ok(input) => input.trim().to_string(),
            Err(_) => panic!("Error creating team abrv")
        
        };  
        match league.new_team(&abrv,&team_name,thread){

            Err(message) => {

                match message{
                    AddTeamError::AbrvTaken => println!("This league already has a team with that abbreviation, please try again"),
                    AddTeamError::NameTaken => println!("This league already has a team with that name, please try again")
                
                };
                //println!("Error {:?}",message);
                prompt_string = "Enter a unique team name";
            },

            Ok(team_string) =>{
                let team_path = path.join(format!("{}.txt", team_name));
                let mut team_info = File::create(team_path)?;
                team_info.write_all(team_string.as_bytes())?;
                result = add_team_check(league, path, thread);
                break;
            }

            
        
        };
        
       /* let team_path = path.join(format!("{}.txt", team_name));
        match team_path.exists() {
            true => {
                println!("A team with that name already exists for this leauge");
                prompt_string = "Enter a unique team name";
            }

            false => {
                let team = league.new_team(&team_name, thread);
                let mut team_info = File::create(team_path)?;
                team_info.write_all(team.to_string().as_bytes())?;
                league.add_team(team);
                result = add_team_check(league, path, thread);
                break;
            }
        }*/
    }

    result
}
//#[tailcall]
fn add_team_check(league: &mut League, path: &Path, thread: &mut ThreadRng) -> std::io::Result<()> {
    let ans = Confirm::new("Would you like to create another team?")
        .with_default(true)
        .prompt();

    match ans {
        Ok(true) => add_new_team(league, path, thread, false),
        Ok(false) => save_league(league, path),
        Err(_) => panic!("Error on add team prompt"),
    }
}

// Take's a league and a path, serializes the league to a json object, which is saved under path.league_info.text
fn save_league(league: &League, path: &Path) -> std::io::Result<()> {
    let serial = serde_json::to_string(&league).unwrap();
    let new_file_path = path.join("league_info.txt");

    let mut league_info = File::create(new_file_path)?;
    league_info.write_all(serial.as_bytes())?;
    Ok(())
}

fn get_league_name() -> String {
    let validator = MinLengthValidator::new(3);
    let name_input = Text::new("Enter the name of the new league")
        .with_validator(validator)
        .prompt();

    match name_input {
        Ok(input) => input.trim().to_string(),
        Err(_) => panic!("Error creating league name"),
    }
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

                let ans = Confirm::new("Would you like to load a different league?")
                    .with_default(true)
                    .prompt();
                //println!("Would you like to load a different league? Y/N");
                match ans {
                    Ok(false) => {
                        result = Ok(());
                        break;
                    }
                    Ok(true) => (),
                    Err(_) => panic!("Erro in the league check"),
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
            let ans = Confirm::new("Would you like to load a different league?")
                .with_default(true)
                .prompt();
            match ans {
                Ok(true) => return league_check(thread),
                Ok(false) => return Ok(()),
                Err(_) => panic!("Error after league is corrupted"),
            }
        }
    };

    println!("League Loaded");
    add_team_check(&mut league, path, thread)
}

//#[tailcall]
fn main() -> std::io::Result<()> {
    let mut r_thread = rand::thread_rng();

    Text::new("Welcome to the Deadball Team generator");

    let starting_options: Vec<&str> = vec![
        "Create a new league.",
        "Add a new team to an existing league.",
    ];

    let starting_choice: Result<&str, InquireError> =
        Select::new("What would you like to do?", starting_options).prompt();

    match starting_choice {
        Ok(choice) => match choice {
            "Create a new league." => create_new_league(&mut r_thread),
            "Add a new team to an existing league." => league_check(&mut r_thread),
            _ => {
                println!("Error with starting choice");
                Ok(())
            }
        },

        Err(_) => {
            println!("Error matching first choice");
            Ok(())
        }
    }
}
